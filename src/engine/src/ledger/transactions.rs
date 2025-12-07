use crate::engine::types::{Amount, ClientId, TransactionId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transaction {
    Inbound(InboundTransaction),
    Outbound(OutboundTransaction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InboundTransaction {
    Settled(TransactionInfo),
    Disputed(TransactionInfo),
    Resolved(TransactionInfo),
    ChargedBack(TransactionInfo),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutboundTransaction {
    Settled(TransactionInfo),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionInfo {
    pub id: TransactionId,
    pub client_id: ClientId,
    pub amount: Amount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Inbound,
    Outbound,
}

#[derive(Debug, Clone, Copy)]
pub enum TransactionStatus {
    Settled,
    Disputed,
    Resolved,
    ChargedBack,
}

#[derive(thiserror::Error, Debug)]
pub enum TransitionError {
    #[error("Invalid transition from {0} to {1}")]
    InvalidTransition(TransactionStatus, TransactionStatus),
    #[error("must be an inbound to transition")]
    InvalidDirection,
}

impl std::fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionStatus::Settled => write!(f, "Settled"),
            TransactionStatus::Disputed => write!(f, "Disputed"),
            TransactionStatus::Resolved => write!(f, "Resolved"),
            TransactionStatus::ChargedBack => write!(f, "ChargedBack"),
        }
    }
}

impl Transaction {
    pub fn new_settled_inbound(id: TransactionId, client_id: ClientId, amount: Amount) -> Self {
        Transaction::Inbound(InboundTransaction::Settled(TransactionInfo {
            id,
            client_id,
            amount,
        }))
    }

    pub fn new_settled_outbound(id: TransactionId, client_id: ClientId, amount: Amount) -> Self {
        Transaction::Outbound(OutboundTransaction::Settled(TransactionInfo {
            id,
            client_id,
            amount,
        }))
    }

    pub fn info(&self) -> &TransactionInfo {
        match self {
            Transaction::Inbound(inbound) => match inbound {
                InboundTransaction::Settled(info)
                | InboundTransaction::Disputed(info)
                | InboundTransaction::Resolved(info)
                | InboundTransaction::ChargedBack(info) => info,
            },
            Transaction::Outbound(outbound) => match outbound {
                OutboundTransaction::Settled(info) => info,
            },
        }
    }

    pub fn status(&self) -> TransactionStatus {
        match self {
            Transaction::Inbound(inbound) => match inbound {
                InboundTransaction::Settled(_) => TransactionStatus::Settled,
                InboundTransaction::Disputed(_) => TransactionStatus::Disputed,
                InboundTransaction::Resolved(_) => TransactionStatus::Resolved,
                InboundTransaction::ChargedBack(_) => TransactionStatus::ChargedBack,
            },
            Transaction::Outbound(outbound) => match outbound {
                OutboundTransaction::Settled(_) => TransactionStatus::Settled,
            },
        }
    }

    pub fn direction(&self) -> Direction {
        match self {
            Transaction::Inbound(_) => Direction::Inbound,
            Transaction::Outbound(_) => Direction::Outbound,
        }
    }

    /// Transitions an inbound transaction through its state machine.
    ///
    /// State Machine:
    /// ```text
    /// ┌─────────┐
    /// │ Settled │
    /// └────┬────┘
    ///      │ dispute
    ///      ▼
    /// ┌──────────┐
    /// │ Disputed │
    /// └────┬─────┘
    ///      │
    ///      ├─────── resolve ────────┐
    ///      │                        ▼
    ///      │                   ┌──────────┐
    ///      │                   │ Resolved │
    ///      │                   └──────────┘
    ///      │
    ///      └──── chargeback ───────┐
    ///                              ▼
    ///                         ┌────────────┐
    ///                         │ ChargedBack│
    ///                         └────────────┘
    /// ```
    ///
    /// Valid transitions:
    /// - Settled → Disputed
    /// - Disputed → Resolved
    /// - Disputed → ChargedBack
    ///
    /// Asumption: A Resolved dispute can't be charged back.
    pub fn transition_inbound(&mut self, status: TransactionStatus) -> Result<(), TransitionError> {
        if self.direction() != Direction::Inbound {
            return Err(TransitionError::InvalidDirection);
        }

        let old_status = self.status();
        match (old_status, status) {
            (TransactionStatus::Settled, TransactionStatus::Disputed) => {
                *self = Transaction::Inbound(InboundTransaction::Disputed(*self.info()));
            }
            (TransactionStatus::Disputed, TransactionStatus::Resolved) => {
                *self = Transaction::Inbound(InboundTransaction::Resolved(*self.info()));
            }
            (TransactionStatus::Disputed, TransactionStatus::ChargedBack) => {
                *self = Transaction::Inbound(InboundTransaction::ChargedBack(*self.info()));
            }
            _ => return Err(TransitionError::InvalidTransition(old_status, status)),
        }

        Ok(())
    }
}
