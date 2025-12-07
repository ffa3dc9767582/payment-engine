use crate::{engine::types::ClientId, ledger::LedgerError, ledger::transactions::TransitionError};

#[derive(thiserror::Error, Debug)]
pub enum EngineError {
    #[error("Invalid associated transaction: {0}")]
    InvalidAssociatedTransaction(&'static str),
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Transaction is in invalid status: {0}")]
    InvalidTransactionStatus(String),
    #[error("Duplicate event")]
    DuplicateEvent,
    #[error("Client {0} account is locked, no further activity is allowed")]
    AccountLocked(ClientId),
    /// Error in events coming from the partner.
    /// Eg. invalid reference to a transaction (invalid direction, not found etc.)
    #[error("Invalid event: {0}")]
    InvalidEvent(&'static str),
    #[error("System error: {0}")]
    SystemError(&'static str),
}

impl From<LedgerError> for EngineError {
    fn from(err: LedgerError) -> Self {
        match err {
            LedgerError::AlreadyExists => EngineError::DuplicateEvent,
            LedgerError::Conflict(message) => EngineError::InvalidEvent(message),
        }
    }
}

impl From<TransitionError> for EngineError {
    fn from(err: TransitionError) -> Self {
        match err {
            TransitionError::InvalidTransition(current, desired) => {
                EngineError::InvalidTransactionStatus(format!(
                    "Operation doesn't apply to this transaction. Transition from {} to {}",
                    current, desired
                ))
            }
            TransitionError::InvalidDirection => {
                // not used, there are better error messages
                EngineError::InvalidEvent("Invalid transaction")
            }
        }
    }
}
