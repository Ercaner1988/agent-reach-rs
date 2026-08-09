mod cli;
mod cmd_execute;

use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Execute {
            task_file,
            output,
            continue_on_error,
            verbose,
        } => {
            cmd_execute::execute_tasks(&task_file, &output, continue_on_error, verbose).await?;
        }
        Commands::Install { .. } => {
            println!("Install subcommand not yet implemented");
            println!("See: https://github.com/Ercaner1988/agent-reach-rs");
        }
        Commands::Configure { .. } => {
            println!("Configure subcommand not yet implemented");
        }
        Commands::Doctor { .. } => {
            println!("Doctor subcommand not yet implemented");
        }
        Commands::Skill { .. } => {
            println!("Skill subcommand not yet implemented");
        }
        Commands::Transcribe { .. } => {
            println!("Transcribe subcommand not yet implemented");
        }
    }

    Ok(())
}
