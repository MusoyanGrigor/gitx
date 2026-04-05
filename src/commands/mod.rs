pub mod undo;
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
    Jump { reference: String },
    /// Undo recent changes safely
    Undo {
        #[command(subcommand)]
        subcommand: UndoSubcommand,

        /// Skip confirmation and proceed (be careful!)
        #[arg(short, long)]
        yes: bool,

        /// Preview actions without executing them
        #[arg(short, long)]
        dry_run: bool,
    },
    /// Interactive or CLI timeline view of commit history
    Timeline {
        /// Filter commits by author
        #[arg(short, long)]
        author: Option<String>,

        /// Filter commits by commit subject/message substring
        #[arg(short, long)]
        message: Option<String>,

        /// Focus on a branch or ref
        #[arg(short, long)]
        branch: Option<String>,

        /// Limit number of commits shown
        #[arg(short, long, default_value = "50")]
        limit: usize,

        /// Show only merge commits
        #[arg(long, conflicts_with = "no_merges")]
        merges: bool,

        /// Hide merge commits
        #[arg(long, conflicts_with = "merges")]
        no_merges: bool,

        /// Use plain CLI timeline output (default for now)
        #[arg(short, long, default_value = "true")]
        cli: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum UndoSubcommand {
    /// Show what can be undone
    Status,
    /// Unstage staged changes (git reset)
    Unstage,
    /// Discard unstaged working tree changes (git restore)
    Discard,
    /// Undo the most recent local commit
    LastCommit {
        /// Move HEAD back but keep changes staged (reset --soft)
        #[arg(short, long)]
        soft: bool,
        /// Undo commit and unstage changes (reset --mixed)
        #[arg(short, long, default_value = "true")] // Safest default
        mixed: bool,
        /// Undo commit and DISCARD all changes (reset --hard) - DANGEROUS
        #[arg(short, long)]
        hard: bool,
    },
    /// Remove untracked files (git clean)
    Clean {
        /// Also remove untracked directories
        #[arg(short = 'd', long)]
        directories: bool,
        /// Also remove ignored files
        #[arg(short = 'x', long)]
        ignored: bool,
    },
    /// Restore a clean repo state (all of the above)
    All {
        /// Also remove untracked files
        #[arg(short, long)]
        clean_untracked: bool,
    },
}
