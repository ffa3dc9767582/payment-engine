//! # Payment Engine
//!
//! A robust, type-safe payment processing engine for handling financial transactions,
//! disputes, and chargebacks with precise decimal arithmetic.
//!
//! ## Overview
//!
//! This crate provides the core business logic for processing payment events including:
//! - **Deposits**: Credit funds to client accounts
//! - **Withdrawals**: Debit funds from client accounts (with balance validation)
//! - **Disputes**: Freeze funds pending investigation
//! - **Resolutions**: Release disputed funds back to available balance
//! - **Chargebacks**: Reverse transactions and lock accounts
//!
//! ## Architecture
//!
//! The engine follows a clean architecture pattern with separation of concerns:
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │          Engine<L: Ledger>          │  ← Main orchestrator
//! │  ┌────────────────────────────────┐ │
//! │  │ accounts: HashMap<ClientId,    │ │  ← Client account state
//! │  │           ClientAccount>       │ │
//! │  └────────────────────────────────┘ │
//! │  ┌────────────────────────────────┐ │
//! │  │ ledger: L (trait impl)         │ │  ← Pluggable storage backend
//! │  └────────────────────────────────┘ │
//! └─────────────────────────────────────┘
//! ```
//!
//! ## Key Features
//!
//! - **Type Safety**: Wrapper types prevent mixing of IDs (`ClientId`, `TransactionId`)
//! - **Precise Arithmetic**: Uses `rust_decimal` for exact financial calculations (4 decimal places)
//! - **State Machine**: Enforces valid transaction state transitions
//! - **Async Ready**: Ledger trait uses async/await for future I/O operations
//! - **Error Classification**: Distinguishes between partner errors (logged) and system errors (fatal)
//!
//! ## Transaction State Machine
//!
//! Inbound transactions (deposits) follow this state machine:
//!
//! ```text
//! Settled → Disputed → Resolved
//!             ↓
//!         ChargedBack
//! ```
//!
//! Outbound transactions (withdrawals) remain in `Settled` state (terminal).
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! use payment_engine::{Engine, Event, ledger::in_memory::InMemoryLedger, types::{ClientId, TransactionId, Amount}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create engine with in-memory storage
//!     let mut engine = Engine::new(InMemoryLedger::new());
//!
//!     // Process a deposit
//!     let event = Event::Deposit {
//!         client_id: ClientId::from(1),
//!         transaction_id: TransactionId::from(1001),
//!         amount: Amount::from_minor(10000), // $100.00
//!     };
//!
//!     engine.apply(event).await?;
//!
//!     // Get account state
//!     for account in engine.accounts() {
//!         println!("Client {} balance: ${}", account.client_id, account.total);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Error Handling
//!
//! The engine uses a dual error classification system:
//!
//! - **Partner Errors**: Invalid data from external sources (continue processing)
//!   - `InsufficientFunds`: Withdrawal exceeds available balance
//!   - `DuplicateEvent`: Transaction ID already exists
//!   - `AccountLocked`: Activity on frozen account
//!
//! - **System Errors**: Internal invariant violations (halt processing)
//!   - Should never occur in normal operation
//!   - Indicates data corruption or logic errors
//!
//! ## Limitations
//!
//! Current implementation has known limitations (documented for transparency):
//!
//! 1. **Non-Atomic Updates**: Ledger and account updates are separate operations
//! 2. **In-Memory Only**: Default implementation doesn't persist to disk
//! 3. **No Compensation**: Real systems would use compensating transactions for reversals
//!
//! These are acceptable for the current use case but should be addressed for production systems.

mod engine;
pub mod ledger;

pub use engine::{ClientAccount, Engine, Event, errors, types};
