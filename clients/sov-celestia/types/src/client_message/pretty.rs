use core::fmt::{Display, Error as FmtError, Formatter};

use super::aggregated_proof::AggregatedProofData;
use super::{AggregatedProof, CodeCommitment, ValidityCondition};
use crate::client_message::aggregated_proof::AggregatedProofPublicInput;
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
            "AggregatedProofData {{ aggregated_proof_public_input: {}, aggregated_proof: {} }}",
            PrettyPublicInput(&self.0.public_input),
            PrettyAggregatedProof(&self.0.aggregated_proof)
        )
    }
}

pub struct PrettyPublicInput<'a>(pub &'a AggregatedProofPublicInput);

impl Display for PrettyPublicInput<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "AggregatedProofPublicInput {{ validity_conditions: {}, initial_slot_number: {}, final_slot_number: {}, initial_da_block_hash: {}, final_da_block_hash: {}, genesis_state_root: {}, input_state_root: {}, post_state_root: {}, code_commitment: {} }}",
            PrettySlice(&self.0.validity_conditions.iter().map(PrettyValidityCondition).collect::<Vec<_>>()),
            self.0.initial_slot_number,
            self.0.final_slot_number,
            PrettySlice(&self.0.initial_da_block_hash),
            PrettySlice(&self.0.final_da_block_hash),
            PrettySlice(&self.0.genesis_state_root),
            PrettySlice(&self.0.input_state_root),
            PrettySlice(&self.0.final_state_root),
            PrettyCodeCommitment(&self.0.code_commitment),
        )
    }
}

pub struct PrettyValidityCondition<'a>(pub &'a ValidityCondition);

impl Display for PrettyValidityCondition<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "ValidityCondition {{ {} }}",
            PrettySlice(self.0.as_slice())
        )
    }
}

pub struct PrettyCodeCommitment<'a>(pub &'a CodeCommitment);

impl Display for PrettyCodeCommitment<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(f, "CodeCommitment {{ {} }}", PrettySlice(self.0.as_slice()))
    }
}

pub struct PrettyAggregatedProof<'a>(pub &'a AggregatedProof);

impl Display for PrettyAggregatedProof<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "AggregatedProof {{ {} }}",
            PrettySlice(self.0.as_slice())
        )
    }
}
