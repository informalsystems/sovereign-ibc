use core::fmt::{Display, Error as FmtError, Formatter};

use super::aggregated_proof::AggregatedProof;
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

pub struct PrettyAggregatedProof<'a>(pub &'a AggregatedProof);

impl Display for PrettyAggregatedProof<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "AggregatedProof {{ public_input: {}, proof: {} }}",
            PrettyPublicInput(&self.0.public_input),
            PrettySlice(&self.0.proof)
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
            self.0.post_state_root
        )
    }
}
