//! Traits and implementations for transaction storage (ledger).
//!
//!

use std::fmt::Debug;

use crate::engine::types::{ClientId, TransactionId};
use crate::ledger::transactions::Transaction;

pub mod in_memory;
pub mod transactions;

pub trait Ledger: Debug {
    /// Add a new transaction to the ledger.
    /// Returns LedgerError::AlreadyExists if the transaction already exists.
    /// Returns LedgerError::Conflict if the transaction exists but has different values or belong to different client..
    fn add(
        &mut self,
        client_id: ClientId,
        transaction: Transaction,
    ) -> impl Future<Output = Result<(), LedgerError>>;

    /// Update an existing transaction in the storage.
    /// Returns LedgerError::NotFound if the transaction does not exist.
    /// Returns LedgerError::Conflict if the transaction belongs to different client.
    fn update(
        &mut self,
        client_id: ClientId,
        transaction: Transaction,
    ) -> impl Future<Output = Result<(), LedgerError>>;

    fn find(
        &self,
        client_id: ClientId,
        transaction_id: TransactionId,
    ) -> impl Future<Output = Result<Option<Transaction>, LedgerError>>;
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum LedgerError {
    #[error("Already exists")]
    AlreadyExists,
    #[error("Conflict: {0}")]
    Conflict(&'static str),
}
