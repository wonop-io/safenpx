use clap::Parser;
use safe_npx::{run, Cli};

/// Parse arguments and print the scaffold report.
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    print!("{}", run(&cli)?);
    Ok(())
}
