use core::fmt::{Display, Error as FmtError, Formatter};

use super::aggregated_proof::AggregatedProofData;
use super::{AggregatedProof, ProofDataInfo};
use crate::client_message::aggregated_proof::PublicInput;
pub struct PrettySlice<'a, T>(pub &'a [T]);

impl<'a, T: Display> Display for PrettySlice<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "[ ")?;
        let mut vec_iterator = self.0.iter().peekable();
        while let Some(element) = vec_iterator.next() {
            write!(f, "{element}")?;
            // If it is not the last element, add separator.
            if vec_iterator.peek().is_some() {
                write!(f, ", ")?;
            }
        }
        write!(f, " ]")
    }
}

pub struct PrettyAggregatedProofData<'a>(pub &'a AggregatedProofData);

impl Display for PrettyAggregatedProofData<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "AggregatedProofData {{ public_input: {}, proof_data_info: {}, aggregated_proof: {} }}",
            PrettyPublicInput(&self.0.public_input),
            PrettyProofDataInfo(&self.0.proof_data_info),
            PrettyAggregatedProof(&self.0.aggregated_proof)
        )
    }
}

pub struct PrettyPublicInput<'a>(pub &'a PublicInput);

impl Display for PrettyPublicInput<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "PublicInput {{ initial_da_block_hash: {:?}, final_da_block_hash: {:?}, input_state_root: {:?}, post_state_root: {:?} }}",
            self.0.initial_da_block_hash,
            self.0.final_da_block_hash,
            self.0.input_state_root,
            self.0.final_state_root
        )
    }
}

pub struct PrettyProofDataInfo<'a>(pub &'a ProofDataInfo);

impl Display for PrettyProofDataInfo<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "ProofDataInfo {{ initial_state_height: {}, final_state_height: {} }}",
            self.0.initial_state_height, self.0.final_state_height
        )
    }
}

pub struct PrettyAggregatedProof<'a>(pub &'a AggregatedProof);

impl Display for PrettyAggregatedProof<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "AggregatedProof {{ {:?} }}", self.0)
    }
}
