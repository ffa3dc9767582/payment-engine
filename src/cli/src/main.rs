use clap::Parser;
use std::path::PathBuf;

use payment_engine::ledger::in_memory::InMemoryLedger;
use payment_engine_cli::app::App;

#[derive(Parser, Debug)]
#[command(author, version, about = "Payment Engine")]
struct Args {
    /// Input file with list of transactions
    file: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let app = App::new();
    let ledger = InMemoryLedger::new();

    app.process(ledger, args.file).await?;

    Ok(())
}
