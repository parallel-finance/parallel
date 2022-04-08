use codec::{Decode, Encode};
use frame_support::RuntimeDebug;
use primitives::{Balance, CurrencyId};
use scale_info::prelude::vec::Vec;
use scale_info::{prelude::vec, TypeInfo};

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo, Default)]
pub struct BridgeToken {
    pub id: CurrencyId,
    pub external: bool,
    pub fee: Balance,
    pub enable: bool,
    pub out_cap: Balance,
    pub out_amount: Balance,
    pub in_cap: Balance,
    pub in_amount: Balance,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum BridgeType {
    // Transfer assets from the current chain to other chains
    BridgeOut = 0,
    // Transfer assets from other chains to the current chain
    BridgeIn = 1,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum ProposalStatus {
    Initiated,
    Approved,
    Rejected,
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, Default, RuntimeDebug, TypeInfo)]
pub struct MaterializeCall<T, E, R> {
    pub bridge_token_id: T,
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

    /// Returns true if the proposal can be removed from storage, otherwise false
    pub fn can_be_cleaned_up(&self, now: E) -> bool {
        self.is_expired(now)
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
