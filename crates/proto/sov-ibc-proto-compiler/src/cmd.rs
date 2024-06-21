use std::path::PathBuf;
use std::process;

use anyhow::Result;
use argh::FromArgs;
use walkdir::WalkDir;

/// App to compile proto files
#[derive(Debug, FromArgs)]
pub struct App {
    #[argh(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, FromArgs)]
#[argh(subcommand)]
pub enum Command {
    Compile(CompileCmd),
}

/// Compile command
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "compile")]
pub struct CompileCmd {
    #[argh(option, short = 'i')]
    /// path to the IBC-Go proto files
    ibc: PathBuf,

    #[argh(option, short = 'o')]
    /// path to output the generated Rust sources into
    out: PathBuf,
}

impl CompileCmd {
    pub fn run(&self) {
        self.compile_protos().unwrap_or_else(|e| {
            eprintln!("[error] failed to compile protos: {e}");
            process::exit(1);
        });

        self.build_pbjson_impls().unwrap_or_else(|e| {
            eprintln!("[error] failed to build pbjson impls: {e}");
            process::exit(1);
        });

        println!("[info] Done!");
    }

    pub fn compile_protos(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "[info] Compiling Sovereign IBC .proto files to Rust into '{}'...",
            self.out.display()
        );

        let root = env!("CARGO_MANIFEST_DIR");

        // Paths
        let proto_paths = [format!("{root}/../definitions")];

        let proto_includes_paths = [
            format!("{}", self.ibc.display()),
            format!("{root}/../definitions"),
        ];

        // List available proto files
        let mut protos: Vec<PathBuf> = vec![];
        for proto_path in &proto_paths {
            println!("Looking for proto files in {proto_path:?}");
            protos.append(
                &mut WalkDir::new(proto_path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.file_type().is_file()
                            && e.path().extension().is_some()
                            && e.path().extension().unwrap() == "proto"
                    })
                    .map(|e| e.into_path())
                    .collect(),
            );
        }

        println!("Found the following protos:");
        // Show which protos will be compiled
        for proto in &protos {
            println!("\t-> {proto:?}");
        }
        println!("[info] Compiling..");

        // List available paths for dependencies
        let includes: Vec<PathBuf> = proto_includes_paths.iter().map(PathBuf::from).collect();

        // Automatically derive a `prost::Name` implementation.
        let mut config = prost_build::Config::new();
        config.enable_type_names();

        tonic_build::configure()
            .compile_well_known_types(true)
            .file_descriptor_set_path(self.out.join("proto_descriptor.bin"))
            .out_dir(self.out.clone())
            .extern_path(".google", "::ibc_proto::google")
            .extern_path(".tendermint", "::tendermint_proto")
            .extern_path(".ics23", "::ibc_proto::ics23")
            .extern_path(".ibc.core", "::ibc_proto::ibc::core")
            .extern_path(
                ".ibc.lightclients.tendermint",
                "::ibc_proto::ibc::lightclients::tendermint",
            )
            .compile_with_config(config, &protos, &includes)?;

        println!("[info] Protos compiled successfully");

        Ok(())
    }

    fn build_pbjson_impls(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[info] Building pbjson Serialize, Deserialize impls...");
        let descriptor_set_path = self.out.join("proto_descriptor.bin");
        let descriptor_set = std::fs::read(descriptor_set_path)?;

        pbjson_build::Builder::new()
            .register_descriptors(&descriptor_set)?
            .out_dir(&self.out)
            .emit_fields()
            .build(&[".ibc.lightclients.sovereign", ".sovereign"])?;

        Ok(())
    }
}
