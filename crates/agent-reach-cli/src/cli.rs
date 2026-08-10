//! CLI subcommands for agent-reach

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "agent-reach")]
#[command(about = "Give your AI Agent eyes to see the entire internet", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// One-shot installer with environment auto-detection
    Install {
        /// Environment: local, server, or auto-detect
        #[arg(long, value_name = "ENV", default_value = "auto")]
        env: String,

        /// Network proxy (http://user:pass@ip:port)
        #[arg(long, value_name = "PROXY")]
        proxy: Option<String>,

        /// Safe mode: skip automatic system changes, show what's needed instead
        #[arg(long)]
        safe: bool,

        /// Dry run: show what would be done without making any changes
        #[arg(long)]
        dry_run: bool,

        /// Comma-separated optional channels to install
        #[arg(long, value_name = "CHANNELS")]
        channels: Option<String>,
    },

    /// Set a config value or auto-extract from browser
    Configure {
        /// Config key to read, set, or unset
        key: Option<String>,

        /// Config value to set
        value: Option<String>,

        /// Remove the selected key instead of reading or setting it
        #[arg(long)]
        unset: bool,

        /// Show config values as JSON
        #[arg(long)]
        json: bool,

        /// Auto-extract cookies from browser
        #[arg(long, value_name = "BROWSER")]
        from_browser: Option<String>,
    },

    /// Check platform availability
    Doctor {
        /// Output machine-readable JSON
        #[arg(long)]
        json: bool,
    },

    /// Manage agent skill registration
    Skill {
        /// Install SKILL.md to agent skill directories
        #[arg(long)]
        install: bool,

        /// Remove SKILL.md from agent skill directories
        #[arg(long)]
        uninstall: bool,
    },

    /// Execute tasks from JSON file (SkillOptOrchestrator integration)
    Execute {
        /// Path to tasks JSON file
        #[arg(long, value_name = "FILE")]
        task_file: String,

        /// Output execution log path (JSON)
        #[arg(long, value_name = "FILE", default_value = "execution_log.json")]
        output: String,

        /// Continue on task failure (default: stop on first error)
        #[arg(long)]
        continue_on_error: bool,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Transcribe audio/video URL or local file (Whisper via Groq/OpenAI)
    Transcribe {
        /// Audio/video URL or local file path
        source: String,

        /// Transcription provider
        #[arg(long, value_name = "PROVIDER", default_value = "auto")]
        provider: String,

        /// Write transcript to file instead of stdout
        #[arg(short, long, value_name = "FILE")]
        output: Option<String>,
    },
}
