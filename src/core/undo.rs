use crate::core::GitRepo;
use crate::models::undo::{CommitUndoInfo, ResetMode, UndoAction, UndoPlan};
use anyhow::{anyhow, Result};
use git2::{ResetType, StatusOptions};

impl GitRepo {
    pub fn plan_undo_status(
        &self,
        include_untracked: bool,
        include_directories: bool,
        include_ignored: bool,
    ) -> Result<UndoPlan> {
        let mut plan = UndoPlan {
            staged_files: Vec::new(),
            unstaged_files: Vec::new(),
            untracked_files: Vec::new(),
            last_commit: None,
            actions: Vec::new(),
        };

        let mut opts = StatusOptions::new();
        opts.include_untracked(include_untracked);
        opts.recurse_untracked_dirs(include_directories);
        opts.include_ignored(include_ignored);
        let statuses = self.repo.statuses(Some(&mut opts))?;

        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("unknown").to_string();
            let s = entry.status();

            if s.is_index_new()
                || s.is_index_modified()
                || s.is_index_deleted()
                || s.is_index_renamed()
                || s.is_index_typechange()
            {
                plan.staged_files.push(path.clone());
            }

            if s.is_wt_modified() || s.is_wt_deleted() || s.is_wt_typechange() {
                plan.unstaged_files.push(path.clone());
            }

            if s.is_wt_new() {
                plan.untracked_files.push(path.clone());
            }
        }

        if let Ok(head) = self.repo.head() {
            if let Some(target) = head.target() {
                if let Ok(commit) = self.repo.find_commit(target) {
                    let mut already_pushed = false;
                    if let Ok(shorthand) = head.shorthand().ok_or(anyhow!("no shorthand")) {
                        if let Ok(branch) =
                            self.repo.find_branch(shorthand, git2::BranchType::Local)
                        {
                            if let Ok(upstream) = branch.upstream() {
                                if let Some(upstream_target) = upstream.get().target() {
                                    if let Ok((ahead, _)) =
                                        self.repo.graph_ahead_behind(target, upstream_target)
                                    {
                                        already_pushed = ahead == 0;
                                    }
                                }
                            }
                        }
                    }

                    plan.last_commit = Some(CommitUndoInfo {
                        hash: target.to_string(),
                        subject: commit.summary().unwrap_or("").to_string(),
                        already_pushed,
                    });
                }
            }
        }

        Ok(plan)
    }

    pub fn execute_undo(&self, plan: &UndoPlan, dry_run: bool) -> Result<()> {
        if dry_run {
            return Ok(());
        }

        for action in &plan.actions {
            match action {
                UndoAction::Unstage(path) => {
                    let head = self.repo.head()?.peel_to_commit()?.into_object();
                    self.repo.reset_default(Some(&head), &[path])?;
                }
                UndoAction::Discard(path) => {
                    let mut checkout_opts = git2::build::CheckoutBuilder::new();
                    checkout_opts.path(path);
                    checkout_opts.force();
                    self.repo.checkout_index(None, Some(&mut checkout_opts))?;
                }
                UndoAction::RemoveUntracked(path) => {
                    let workdir = self.repo.workdir().ok_or_else(|| anyhow!("No workdir"))?;
                    let full_path = workdir.join(path);
                    if full_path.is_dir() {
                        std::fs::remove_dir_all(full_path)?;
                    } else if full_path.is_file() {
                        std::fs::remove_file(full_path)?;
                    }
                }
                UndoAction::ResetCommit { hash: _, mode } => {
                    let head = self.repo.head()?;
                    let current_commit = head.peel_to_commit()?;
                    let parent = current_commit
                        .parent(0)
                        .map_err(|_| anyhow!("Commit has no parent to reset to"))?;

                    let reset_type = match mode {
                        ResetMode::Soft => ResetType::Soft,
                        ResetMode::Mixed => ResetType::Mixed,
                        ResetMode::Hard => ResetType::Hard,
                    };
                    self.repo.reset(&parent.into_object(), reset_type, None)?;
                }
            }
        }

        Ok(())
    }
}
