use codec::{Decode, Encode};
use frame_support::RuntimeDebug;
// use primitives::{AccountId, Balance, CurrencyId};
use scale_info::prelude::vec::Vec;
use scale_info::{prelude::vec, TypeInfo};

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum ProposalStatus {
    Initiated,
    Approved,
    Rejected,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct MaterializeCall<T, E, R> {
    pub currency_id: T,
    pub to: E,
    pub amount: R,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Proposal<T, E> {
    pub votes_for: Vec<T>,
    pub votes_against: Vec<T>,
    pub status: ProposalStatus,
    pub expiry: E,
}

impl<T: PartialEq, E: PartialOrd + Default> Proposal<T, E> {
    /// Attempts to mark the proposal as approve or rejected.
    /// Returns true if the status changes from active.
    pub fn try_to_complete(&mut self, threshold: u32, total: u32) -> ProposalStatus {
        if self.votes_for.len() >= threshold as usize {
            self.status = ProposalStatus::Approved;
            ProposalStatus::Approved
        } else if total >= threshold && self.votes_against.len() as u32 + threshold > total {
            self.status = ProposalStatus::Rejected;
            ProposalStatus::Rejected
        } else {
            ProposalStatus::Initiated
        }
    }

    /// Returns true if the proposal has been rejected or approved, otherwise false.
    pub fn is_complete(&self) -> bool {
        self.status != ProposalStatus::Initiated
    }

    /// Returns true if `who` has voted for or against the proposal
    pub fn has_voted(&self, who: &T) -> bool {
        self.votes_for.contains(who) || self.votes_against.contains(who)
    }

    /// Return true if the expiry time has been reached
    pub fn is_expired(&self, now: E) -> bool {
        self.expiry <= now
    }
}

impl<T, E: Default> Default for Proposal<T, E> {
    fn default() -> Self {
        Self {
            votes_for: vec![],
            votes_against: vec![],
            status: ProposalStatus::Initiated,
            expiry: E::default(),
        }
    }
}
