use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<GitXCommand>,

    /// Raw git arguments to forward if no command matches
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub raw_args: Vec<String>,
}

#[derive(Subcommand, Debug)]
pub enum GitXCommand {
    /// Explore the repository visually in TUI
    Tree {
        /// Filter by branch, author, or message
        #[arg(short, long)]
        filter: Option<String>,

        /// Use plain CLI tree output
        #[arg(short, long)]
        cli: bool,

        /// Limit number of commits shown
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },
    /// Compare two branches
    Compare {
        branch1: String,
        branch2: String,
        /// Show detailed diff summary
        #[arg(short, long)]
        detail: bool,
    },
    /// Jump to a specific commit, tag, or branch
    Jump {
        reference: String,
    },
    /// Sequential commit history viewer (MVP Stub)
    Timeline,
}
