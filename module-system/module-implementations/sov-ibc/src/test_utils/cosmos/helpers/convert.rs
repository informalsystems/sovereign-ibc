use ibc::core::ics23_commitment::merkle::MerkleProof;
use ibc_proto::ibc::core::commitment::v1::MerkleProof as RawMerkleProof;
use ibc_proto::ics23::CommitmentProof;
use tendermint::merkle::proof::ProofOps;

pub fn convert_tm_to_ics_merkle_proof(tm_proof: &ProofOps) -> MerkleProof {
    let mut proofs = Vec::new();

    for op in &tm_proof.ops {
        let mut parsed = CommitmentProof { proof: None };

        prost::Message::merge(&mut parsed, op.data.as_slice()).unwrap();

        proofs.push(parsed);
    }

    MerkleProof::from(RawMerkleProof { proofs })
}
