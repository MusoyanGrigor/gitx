use anyhow::{Result, anyhow};
use std::io::{self, Write};
use crate::models::undo::{UndoPlan, UndoAction, ResetMode};
use crate::commands::UndoSubcommand;
use crate::core::GitRepo;
use crossterm::style::Stylize;

pub fn handle_undo(repo: GitRepo, sub: UndoSubcommand, yes: bool, dry_run: bool) -> Result<()> {
    // Determine status options based on subcommand
    let (include_untracked, include_dirs, include_ignored) = match sub {
        UndoSubcommand::Clean { directories, ignored } => (true, directories, ignored),
        UndoSubcommand::All { clean_untracked } => (clean_untracked, clean_untracked, false),
        UndoSubcommand::Status => (true, true, true), // Show all in status
        _ => (false, false, false),
    };

    let mut plan = repo.plan_undo_status(include_untracked, include_dirs, include_ignored)?;
    
    match sub {
        UndoSubcommand::Status => {
            print_undo_status(&plan);
            return Ok(());
        }
        UndoSubcommand::Unstage => {
            plan.actions = plan.staged_files.iter().map(|f| UndoAction::Unstage(f.clone())).collect();
        }
        UndoSubcommand::Discard => {
            plan.actions = plan.unstaged_files.iter().map(|f| UndoAction::Discard(f.clone())).collect();
        }
        UndoSubcommand::LastCommit { soft, mixed: _, hard } => {
            if let Some(ref info) = plan.last_commit {
                let mode = if hard { ResetMode::Hard } 
                           else if soft { ResetMode::Soft }
                           else { ResetMode::Mixed }; 
                plan.actions.push(UndoAction::ResetCommit { hash: info.hash.clone(), mode });
            } else {
                return Err(anyhow!("No last commit to undo."));
            }
        }
        UndoSubcommand::Clean { .. } => {
            plan.actions = plan.untracked_files.iter().map(|f| UndoAction::RemoveUntracked(f.clone())).collect();
        }
        UndoSubcommand::All { .. } => {
            for f in &plan.staged_files { plan.actions.push(UndoAction::Unstage(f.clone())); }
            for f in &plan.unstaged_files { plan.actions.push(UndoAction::Discard(f.clone())); }
            for f in &plan.untracked_files { plan.actions.push(UndoAction::RemoveUntracked(f.clone())); }
        }
    }

    if plan.actions.is_empty() {
        println!("{}", "Nothing to undo.".dark_grey());
        return Ok(());
    }

    // Preview
    println!("{}", "--- [ Undo Plan Preview ] ---".bold());
    for action in &plan.actions {
        match action {
            UndoAction::Unstage(f) => println!("  {} unstage {}", "[-]".yellow(), f),
            UndoAction::Discard(f) => println!("  {} discard changes in {}", "[!]".red(), f),
            UndoAction::RemoveUntracked(f) => println!("  {} remove untracked {}", "[x]".red(), f),
            UndoAction::ResetCommit { hash, mode } => {
                let hash_short = if hash.len() > 7 { &hash[..7] } else { hash };
                println!("  {} reset last commit ({}) in {:?}", "[R]".red(), hash_short, mode);
            }
        }
    }

    if dry_run {
        println!("\n{}", "Dry run: No changes were made.".cyan());
        return Ok(());
    }

    // Confirmation
    if !yes {
        print!("\n{} Proceed with undo? [y/N]: ", "CAUTION:".yellow().bold());
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim().to_lowercase() != "y" {
            println!("Canceled.");
            return Ok(());
        }
    }

    repo.execute_undo(&plan, false)?;
    println!("\n{}", "Undo completed successfully.".green().bold());
    
    Ok(())
}

fn print_undo_status(plan: &UndoPlan) {
    println!("{}", "--- [ Undo Status ] ---".bold().cyan());
    
    let is_clean = plan.staged_files.is_empty() 
        && plan.unstaged_files.is_empty() 
        && plan.untracked_files.is_empty() 
        && plan.last_commit.is_none();

    if is_clean {
        println!("Nothing can be undone. Repository is clean.");
        return;
    }

    if !plan.staged_files.is_empty() {
        println!("\n{}:", "Staged changes".yellow().bold());
        for f in &plan.staged_files { println!("  {}", f); }
    }

    if !plan.unstaged_files.is_empty() {
        println!("\n{}:", "Unstaged changes (tracked)".red().bold());
        for f in &plan.unstaged_files { println!("  {}", f); }
    }

    if !plan.untracked_files.is_empty() {
        println!("\n{}:", "Untracked files".red());
        for f in &plan.untracked_files { println!("  {}", f); }
    }

    if let Some(ref info) = plan.last_commit {
        println!("\n{}:", "Last commit".cyan().bold());
        let hash_short = if info.hash.len() > 7 { &info.hash[..7] } else { &info.hash };
        println!("  {} - {}", hash_short, info.subject);
        if info.already_pushed {
            println!("  {}", "WARNING: This commit appears to be already pushed to a remote.".red().italic());
        }
    }
}
