//! Execute subcommand — run tasks from JSON file (SkillOptOrchestrator integration)

use agent_reach_channels::WebChannel;
use agent_reach_core::{Channel, Config};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::Instant;

/// Task definition from SkillOptOrchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task identifier
    pub id: String,
    /// Platform/channel name (e.g., "web", "youtube", "twitter")
    pub channel: String,
    /// Action to perform (e.g., "read", "search", "fetch")
    pub action: String,
    /// Arguments for the action (e.g., URL, query string, user ID)
    pub args: Vec<String>,
    /// Optional task-specific metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Task ID
    pub task_id: String,
    /// Success status
    pub success: bool,
    /// Platform that executed the task
    pub channel: String,
    /// Backend that successfully executed (if success)
    pub backend: Option<String>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Result data (if success)
    pub output: Option<serde_json::Value>,
    /// Error message (if failure)
    pub error: Option<String>,
}

/// Execution log — list of task results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLog {
    /// Total execution time in milliseconds
    pub total_duration_ms: u64,
    /// Task results
    pub results: Vec<TaskResult>,
    /// Overall success (all tasks succeeded)
    pub success: bool,
}

/// Execute tasks from JSON file
pub async fn execute_tasks(
    task_file: &str,
    output_file: &str,
    continue_on_error: bool,
    verbose: bool,
) -> Result<()> {
    let start = Instant::now();

    // Load tasks from JSON file
    let tasks_json = fs::read_to_string(task_file)
        .with_context(|| format!("Failed to read task file: {}", task_file))?;

    let tasks: Vec<Task> = serde_json::from_str(&tasks_json)
        .with_context(|| format!("Failed to parse tasks JSON from {}", task_file))?;

    if verbose {
        println!("Loaded {} tasks from {}", tasks.len(), task_file);
    }

    // Load config
    let config = Config::load().unwrap_or_default();

    // Execute tasks
    let mut results = Vec::new();
    let mut overall_success = true;

    for task in tasks {
        if verbose {
            println!("Executing task {}: {} {} {:?}", task.id, task.channel, task.action, task.args);
        }

        let task_result = execute_single_task(&task, &config, verbose).await;

        if !task_result.success {
            overall_success = false;
            if !continue_on_error {
                results.push(task_result);
                break;
            }
        }

        results.push(task_result);
    }

    // Build execution log
    let log = ExecutionLog {
        total_duration_ms: start.elapsed().as_millis() as u64,
        results,
        success: overall_success,
    };

    // Write execution log to JSON file
    let log_json = serde_json::to_string_pretty(&log)
        .context("Failed to serialize execution log")?;

    fs::write(output_file, log_json)
        .with_context(|| format!("Failed to write execution log to {}", output_file))?;

    if verbose {
        println!("Execution log written to {}", output_file);
    }

    if !overall_success {
        anyhow::bail!("Some tasks failed (see execution log)");
    }

    Ok(())
}

/// Execute a single task
async fn execute_single_task(task: &Task, config: &Config, verbose: bool) -> TaskResult {
    let start = Instant::now();

    // Route to appropriate channel
    let result = match task.channel.as_str() {
        "web" => {
            let channel = WebChannel::new();
            channel.execute(&task.action, &task.args, config).await
        }
        // TODO: Add other channels (youtube, rss, twitter, etc.)
        _ => {
            return TaskResult {
                task_id: task.id.clone(),
                success: false,
                channel: task.channel.clone(),
                backend: None,
                duration_ms: start.elapsed().as_millis() as u64,
                output: None,
                error: Some(format!("Channel '{}' not implemented yet", task.channel)),
            };
        }
    };

    match result {
        Ok(output) => {
            if verbose {
                println!("  ✓ {} via {} ({}ms)", task.id, output.backend, output.duration_ms);
            }
            TaskResult {
                task_id: task.id.clone(),
                success: true,
                channel: output.platform,
                backend: Some(output.backend),
                duration_ms: output.duration_ms,
                output: Some(output.data),
                error: None,
            }
        }
        Err(e) => {
            if verbose {
                println!("  ✗ {}: {}", task.id, e);
            }
            TaskResult {
                task_id: task.id.clone(),
                success: false,
                channel: task.channel.clone(),
                backend: None,
                duration_ms: start.elapsed().as_millis() as u64,
                output: None,
                error: Some(e.to_string()),
            }
        }
    }
}
