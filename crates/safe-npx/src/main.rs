use safe_npx::{run_with_exit_code, Cli};

/// Parse arguments and print the scaffold report.
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let output = run_with_exit_code(&cli)?;
    print!("{}", output.stdout);
    if output.exit_code != 0 {
        std::process::exit(output.exit_code);
    }
    Ok(())
}
