use alloc::vec::Vec;
use core::fmt::{Display, Error as FmtError, Formatter};

use celestia_types::ExtendedHeader;
use tendermint::validator::Set as ValidatorSet;

use crate::types::client_message::aggregate_snark::AggregateSNARK;
use crate::types::client_message::celestia_header::CelestiaHeader;
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

pub struct PrettyCelestiaHeader<'a>(pub &'a CelestiaHeader);

impl Display for PrettyCelestiaHeader<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "CelestiaHeader {{ extended_header: {}, trusted_height: {}, trusted_next_validator_set: {} }}",
            PrettyExtendedHeader(&self.0.extended_header),
            self.0.trusted_height,
            PrettyValidatorSet(&self.0.trusted_next_validator_set)
        )
    }
}

pub struct PrettyExtendedHeader<'a>(pub &'a ExtendedHeader);

impl Display for PrettyExtendedHeader<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "ExtendedHeader {{ header: {{ chain_id: {}, height: {} }}, commit: {{ height: {} }} }}",
            self.0.header.chain_id, self.0.header.height, self.0.commit.height
        )
    }
}

pub struct PrettyValidatorSet<'a>(pub &'a ValidatorSet);

impl Display for PrettyValidatorSet<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        let validator_addresses: Vec<_> = self
            .0
            .validators()
            .iter()
            .map(|validator| validator.address)
            .collect();
        if let Some(proposer) = self.0.proposer() {
            match &proposer.name {
                Some(name) => write!(f, "PrettyValidatorSet {{ validators: {}, proposer: {}, total_voting_power: {} }}", PrettySlice(&validator_addresses), name, self.0.total_voting_power()),
                None =>  write!(f, "PrettyValidatorSet {{ validators: {}, proposer: None, total_voting_power: {} }}", PrettySlice(&validator_addresses), self.0.total_voting_power()),
            }
        } else {
            write!(
                f,
                "PrettyValidatorSet {{ validators: {}, proposer: None, total_voting_power: {} }}",
                PrettySlice(&validator_addresses),
                self.0.total_voting_power()
            )
        }
    }
}

pub struct PrettyAggregateSNARK<'a>(pub &'a AggregateSNARK);

impl Display for PrettyAggregateSNARK<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        write!(
            f,
            "AggregateSNARK {{ start_da_hash: {:?}, end_da_hash: {:?}, input_state_root: {:?}, output_state_root: {:?} }}",
            self.0.start_da_hash,
            self.0.end_da_hash,
            self.0.input_state_root,
            self.0.output_state_root
        )
    }
}
