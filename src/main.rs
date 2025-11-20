use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod ai;
mod context;
mod core;
mod tui;

#[derive(Parser)]
#[command(name = "aether")]
#[command(about = "The Neural Fabric for your Shell", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Mode to run in (lens, pipe, sentinel)
    #[arg(long, default_value = "lens")]
    mode: String,

    /// Current buffer content (for lens mode)
    #[arg(long, default_value = "")]
    buffer: String,

    /// Cursor position in the buffer
    #[arg(long, default_value = "0")]
    cursor_pos: usize,
}

#[derive(Subcommand)]
enum Commands {
    /// Inject shell integration script
    Inject {
        /// Shell type (zsh, bash, fish)
        #[arg(value_name = "SHELL")]
        shell: String,
    },
    /// Run in lens mode (Raycast-like overlay)
    Lens,
    /// Run in pipe mode (process stdin)
    Pipe,
    /// Run in sentinel mode (error analysis)
    Sentinel,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aether=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Inject { shell }) => {
            inject_shell_integration(shell)?;
        }
        Some(Commands::Lens) | None if cli.mode == "lens" => {
            tui::run_lens_mode(cli.buffer, cli.cursor_pos).await?;
        }
        Some(Commands::Pipe) | None if cli.mode == "pipe" => {
            run_pipe_mode().await?;
        }
        Some(Commands::Sentinel) | None if cli.mode == "sentinel" => {
            run_sentinel_mode().await?;
        }
        _ => {
            eprintln!("Unknown mode or command");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn inject_shell_integration(shell: &str) -> Result<()> {
    match shell {
        "zsh" => {
            println!("{}", include_str!("shell_integration/zsh.sh"));
        }
        "bash" => {
            println!("{}", include_str!("shell_integration/bash.sh"));
        }
        _ => {
            anyhow::bail!("Unsupported shell: {}. Use 'zsh' or 'bash'", shell);
        }
    }
    Ok(())
}

async fn run_pipe_mode() -> Result<()> {
    use std::io::{self, Read};

    // Read from stdin
    let mut stdin_data = String::new();
    io::stdin().read_to_string(&mut stdin_data)?;

    if stdin_data.is_empty() {
        eprintln!("Error: No input provided via stdin");
        eprintln!("Usage: cat file.txt | aether --mode pipe");
        eprintln!("   or: echo 'data' | ae 'instruction'");
        std::process::exit(1);
    }

    // TODO: Phase 3 - Process with AI
    // For now, just echo back with a message
    println!("Received {} bytes of input", stdin_data.len());
    println!("Data preview: {}", &stdin_data.chars().take(100).collect::<String>());
    println!("\nNote: AI processing will be implemented in Phase 3");
    println!("For now, pipe mode just echoes your input.");

    Ok(())
}

async fn run_sentinel_mode() -> Result<()> {
    // TODO: Phase 4 - Implement sentinel mode
    eprintln!("Sentinel mode not yet implemented");
    Ok(())
}
