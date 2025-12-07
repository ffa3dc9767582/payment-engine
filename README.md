# Payment Engine

A payment processing engine for handling deposits, withdrawals, and dispute resolution with client account management.

## Usage

Tested with Rust version `1.91.1`. A `shell.nix` is provided for Nix users (Linux & macOS).

Various example CSV files available in the `example_inputs/[error|success]` directory.

```bash
# (optional) if you use nix and don't have rust installed
nix-shell

cargo run -- example_inputs/success/dispute_chargeback.cs
cargo run -- example_inputs/errors/locked_account_activity.cs

# Run all examples
make run-all

# or Using docker
make docker-build
make docker-run FILE=example_inputs/success/dispute_chargeback.csv
```

## Architecture

Crates are organized as a Rust workspace with clear separation between the CLI application and the core engine library.

```
┌─────────────────────────────────────────────────────────────────┐
│                       CLI Application                           │
│                         (src/cli)                               │
│                                                                 │
│  • Command-line Argument Parsing                                │
│  • CSV Input streaming /Output                                  │
│  • Error Classification (Partner vs System)                     │
│  • Integration tests for the entire system                      │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         │ uses
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Engine (lib)                               │
│                      (src/engine)                               │
│                                                                 │
│  • Event Processing Orchestration                               │
│  • Account Management                                           │
│  • Transaction Application Logic                                │
│  • Generic over Ledger: Engine<L: Ledger>                       │
└─────┬────────────────────────────────────────────┬──────────────┘
      │                                            │
      │ uses                                       │ uses
      ▼                                            ▼
┌──────────────────────────┐          ┌────────────────────────────┐
│    Domain Layer          │          │    Storage Layer           │
│                          │          │                            │
│  • Events (events.rs)    │          │  • Ledger Trait            │
│  • Types (types.rs)      │          │    (ledger.rs)             │
│    - ClientId            │          │                            │
│    - TransactionId       │          │  • InMemoryLedger          │
│    - Amount (Decimal)    │          │    (ledger/in_memory.rs)   │
│  • Accounts              │          │                            │
│    (accounts.rs)         │          │  • Transactions            │
│  • Errors (errors.rs)    │          │    (ledger/transactions.rs)│
└──────────────────────────┘          └────────────────────────────┘
                                                   │
                                                   │ implements
                                                   ▼
                                      ┌────────────────────────────┐
                                      │  Transaction State Machine │
                                      │                            │
                                      │  Inbound:                  │
                                      │    Settled                 │
                                      │      └→ Disputed           │
                                      │          | └→ Resolved     │
                                      │          └→ ChargedBack    │
                                      │                            │
                                      │  Outbound:                 │
                                      │    Settled (terminal)      │
                                      └────────────────────────────┘
```

### Benefits

1. Separation of Concerns: `engine` crate uses minimal dependency and expose the Engine and Ledger;
   1. `cli` crate handles I/O related to csv file. It can stream large CSV files efficiently.
   2. There could be more ports in the future (eg. web server) that uses the `engine` crate.
2. Generic Storage: `Engine<L: Ledger>` allows pluggable storage implementations, an in-memory ledger is provided.
3. Async-Ready: Ledger trait use async/await to allow real storage impls (eg. postgres)
4. Type Safety: Wrapper types prevent mixing ClientIds with TransactionIds
5. Error Separation: Partner errors (bad data) are logged; system errors (invariants) panic
6. Comprehensive Testing: Unit tests for in-memory ledger and integration tests for the CLI that implicitly tests the engine. Integration tests is the best way to cover a lot of ground in short time, that is why I opted for integration test. 

## Note / Assumptions
1. A transaction must be disputed before `resolve` and `chargeback` can be applied. See the docs for `transition_inbound` to see the state machine. 
2. Dispute is only allowed for an inbound transaction (deposit).
3. Dispute can fail if there are insufficient funds available in the account, this is to prevent negative balances.
4. Once a client account is locked (due to chargeback), no further transactions are allowed. 
5. Error handling:
   1. Any invalid input due as a result of partner error is skipped. Error message will be written to stderr. 
   2. Data with negative amounts is considered invalid and ignored.
   3. The `engine` library however expose detailed error messages. The CLI groups them into partner error (ignore) and system error (panic!).
6. For simplicity, this exercise does not include idempotency checks. Duplicate transaction IDs will cause errors. Out-of-order or concurrent events are also not handled, as they are not an issue in a single-threaded appp with in-memory storage.

## Error Display

The CLI writes error messages into stderr to ensure the Engine output can be written to a file, while giving meaningful error messages in the stderr.

```
❯ cargo run -q -- example_inputs/errors/chargeback_without_dispute.csv > accounts.csv
Partner Data Error for TxId: 1, ClientId: 1: Transaction is in invalid status: Operation doesn't apply to this transaction. Transition from Settled to ChargedBack

❯ cat accounts.csv
client,available,held,total,locked
1,3,0,3,false
```
