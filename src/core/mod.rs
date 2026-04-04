pub mod undo;

use anyhow::{Result, anyhow};
use git2::{Repository, Commit, Oid, BranchType, Sort};
use crate::models::{CommitInfo, ComparisonResult, LabelInfo};

pub struct GitRepo {
    pub repo: Repository,
}

impl GitRepo {
    pub fn open_default() -> Result<Self> {
        let repo = Repository::open_from_env()?;
        Ok(Self { repo })
    }

    pub fn get_commits(&self, limit: usize) -> Result<Vec<CommitInfo>> {
        let mut walk = self.repo.revwalk()?;
        walk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?;
        walk.push_head().map_err(|_| anyhow!("Could not push HEAD. Empty repo?"))?;

        let mut commits = Vec::new();
        for id in walk.take(limit) {
            let id = id?;
            if let Ok(commit) = self.repo.find_commit(id) {
                commits.push(self.convert_commit(&commit)?);
            }
        }
        Ok(commits)
    }

    pub fn filter_commits(&self, query: &str) -> Result<Vec<CommitInfo>> {
        let mut walk = self.repo.revwalk()?;
        walk.set_sorting(Sort::TIME)?;
        walk.push_head()?;

        let mut results = Vec::new();
        let query_lower = query.to_lowercase();
        for id in walk {
            let id = id?;
            let commit = self.repo.find_commit(id)?;
            let info = self.convert_commit(&commit)?;
            if info.subject.to_lowercase().contains(&query_lower) || 
               info.author.to_lowercase().contains(&query_lower) ||
               info.hash.to_lowercase().starts_with(&query_lower) {
                results.push(info);
                if results.len() >= 100 { break; }
            }
        }
        Ok(results)
    }

    pub fn resolve_ref(&self, reference: &str) -> Result<String> {
        let obj = self.repo.revparse_single(reference)?;
        let commit = obj.peel_to_commit()?;
        Ok(commit.id().to_string())
    }

    pub fn compare(&self, branch1: &str, branch2: &str) -> Result<ComparisonResult> {
        let obj1 = self.repo.revparse_single(branch1)?;
        let obj2 = self.repo.revparse_single(branch2)?;
        
        let c1 = obj1.as_commit().ok_or_else(|| anyhow!("Ref {} is not a commit", branch1))?;
        let c2 = obj2.as_commit().ok_or_else(|| anyhow!("Ref {} is not a commit", branch2))?;

        let base_oid = self.repo.merge_base(c1.id(), c2.id())?;
        
        // Find commits in branch1 NOT in branch2
        let mut walk1 = self.repo.revwalk()?;
        walk1.push(c1.id())?;
        walk1.hide(c2.id())?;
        let mut a_only = Vec::new();
        for id in walk1 {
            a_only.push(self.convert_commit(&self.repo.find_commit(id?)?)?);
        }

        // Find commits in branch2 NOT in branch1
        let mut walk2 = self.repo.revwalk()?;
        walk2.push(c2.id())?;
        walk2.hide(c1.id())?;
        let mut b_only = Vec::new();
        for id in walk2 {
            b_only.push(self.convert_commit(&self.repo.find_commit(id?)?)?);
        }

        Ok(ComparisonResult {
            base_hash: base_oid.to_string(),
            unique_to_a: a_only,
            unique_to_b: b_only,
        })
    }

    fn convert_commit(&self, commit: &Commit) -> Result<CommitInfo> {
        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown").to_string();
        let time = commit.time().seconds();

        let labels = self.get_labels_for_commit(commit.id())?;
        let parents = commit.parent_ids().map(|id| id.to_string()).collect();

        Ok(CommitInfo {
            hash: commit.id().to_string(),
            author: author_name,
            date: time,
            subject: commit.summary().unwrap_or("").to_string(),
            body: commit.body().map(|s| s.to_string()),
            labels,
            parents,
        })
    }

    fn get_labels_for_commit(&self, oid: Oid) -> Result<Vec<LabelInfo>> {
        let mut labels = Vec::new();

        if let Ok(head) = self.repo.head() {
            if let Some(target) = head.target() {
                if target == oid {
                    let name = head.shorthand().unwrap_or("HEAD").to_string();
                    labels.push(LabelInfo::Head(name));
                }
            }
        }

        for branch in self.repo.branches(None)? {
            let (b, b_type) = branch?;
            if let Some(target) = b.get().target() {
                if target == oid {
                    let name = b.name()?.unwrap_or("").to_string();
                    match b_type {
                        BranchType::Local => labels.push(LabelInfo::LocalBranch(name)),
                        BranchType::Remote => labels.push(LabelInfo::RemoteBranch(name)),
                    }
                }
            }
        }

        self.repo.tag_foreach(|id, name| {
            if id == oid {
                let name_str = std::str::from_utf8(name).unwrap_or("").replace("refs/tags/", "");
                labels.push(LabelInfo::Tag(name_str));
            }
            true
        })?;

        Ok(labels)
    }
}
