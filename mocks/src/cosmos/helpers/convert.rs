use ibc_core::commitment_types::merkle::MerkleProof;
use ibc_core::commitment_types::proto::ics23::CommitmentProof;
use tendermint::merkle::proof::ProofOps;

pub fn convert_tm_to_ics_merkle_proof(tm_proof: &ProofOps) -> MerkleProof {
    let mut proofs = Vec::new();

    for op in &tm_proof.ops {
        let mut parsed = CommitmentProof { proof: None };

        prost::Message::merge(&mut parsed, op.data.as_slice()).unwrap();

        proofs.push(parsed);
    }

    MerkleProof { proofs }
}
