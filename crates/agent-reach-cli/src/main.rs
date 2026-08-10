mod cli;
mod cmd_configure;
mod cmd_doctor;
mod cmd_execute;
mod cmd_install;
mod cmd_skill;
mod cmd_transcribe;

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
        Commands::Install {
            env,
            proxy,
            safe,
            dry_run,
            channels,
        } => {
            cmd_install::install(env, proxy, safe, dry_run, channels).await?;
        }
        Commands::Configure {
            key,
            value,
            unset,
            json,
            from_browser,
        } => {
            cmd_configure::configure(key, value, unset, json, from_browser).await?;
        }
        Commands::Doctor { json } => {
            cmd_doctor::doctor(json).await?;
        }
        Commands::Skill { install, uninstall } => {
            cmd_skill::skill(install, uninstall).await?;
        }
        Commands::Transcribe {
            source,
            provider,
            output,
        } => {
            cmd_transcribe::transcribe(source, provider, output).await?;
        }
    }

    Ok(())
}
