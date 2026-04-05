mod core;
mod commands;
mod forwarding;
mod models;
mod tui;
mod utils;

use anyhow::Result;
use clap::Parser;
use crate::commands::{Cli, GitXCommand};
use crate::core::GitRepo;
use crate::forwarding::forward_to_git;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() -> Result<()> {
    // Setup tracing/logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    match cli.command {
        Some(GitXCommand::Tree { filter, cli, limit }) => {
            let repo = GitRepo::open_default()?;
            if !cli {
                tui::run_tree_explorer(repo, filter, None)?;
            } else {
                let commits = if let Some(ref q) = filter {
                    repo.filter_commits(q)?
                } else {
                    repo.get_commits(limit)?
                };
                let mut renderer = crate::utils::graph_renderer::GraphRenderer::new();
                renderer.render(&commits);
            }
        }
        Some(GitXCommand::Compare { branch1, branch2, detail }) => {
            let repo = GitRepo::open_default()?;
            let result = repo.compare(&branch1, &branch2)?;
            println!("--- [ GitX Comparison ] ---");
            println!("Merge Base: {}", result.base_hash);
            println!("Unique to {}: {} commits", branch1, result.unique_to_a.len());
            println!("Unique to {}: {} commits", branch2, result.unique_to_b.len());
            if detail {
                for c in &result.unique_to_a {
                    println!("[{}] - {} - {}", &c.hash[..7], c.author, c.subject);
                }
            }
        }
        Some(GitXCommand::Jump { reference }) => {
            let repo = GitRepo::open_default()?;
            // Resolve ref and scroll to it in the full tree
            tui::run_tree_explorer(repo, None, Some(reference))?;
        }
        Some(GitXCommand::Timeline) => {
            println!("Timeline playback mode is coming soon in v0.2.0");
        }
        Some(GitXCommand::Undo { subcommand, yes, dry_run }) => {
            let repo = GitRepo::open_default()?;
            crate::commands::undo::handle_undo(repo, subcommand, yes, dry_run)?;
        }
        None => {
            // No subcommand provided. Check for raw args for forwarding.
            if !cli.raw_args.is_empty() {
                forward_to_git(cli.raw_args)?;
            } else {
                // No command, no args: show help
                use clap::CommandFactory;
                Cli::command().print_help()?;
            }
        }
    }

    Ok(())
}
