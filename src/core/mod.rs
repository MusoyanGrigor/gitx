pub mod undo;

use crate::models::{CommitInfo, ComparisonResult, LabelInfo};
use anyhow::{anyhow, Result};
use git2::{BranchType, Commit, Oid, Repository, Sort};

#[derive(Default, Debug)]
pub struct CommitFilter {
    pub author: Option<String>,
    pub message: Option<String>,
    pub query: Option<String>, // Generic query from 'tree --filter'
    pub branch: Option<String>,
    pub limit: usize,
    pub only_merges: bool,
    pub no_merges: bool,
}

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
        walk.push_head()
            .map_err(|_| anyhow!("Could not push HEAD. Empty repo?"))?;

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
        self.filter_commits_ext(CommitFilter {
            query: Some(query.to_string()),
            limit: 100,
            ..Default::default()
        })
    }

    pub fn filter_commits_ext(&self, filter: CommitFilter) -> Result<Vec<CommitInfo>> {
        let mut walk = self.repo.revwalk()?;
        walk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?;

        if let Some(ref b) = filter.branch {
            let obj = self.repo.revparse_single(b)?;
            walk.push(obj.id())?;
        } else {
            walk.push_head()
                .map_err(|_| anyhow!("Could not push HEAD. Empty repo?"))?;
        }

        let mut matched_oids = Vec::new();
        let mut matched_set = std::collections::HashSet::new();

        // 1. Find all matching commits first
        for id in walk {
            let id = id?;
            let commit = self.repo.find_commit(id)?;

            let mut matches = true;

            if let Some(ref q) = filter.query {
                let q_l = q.to_lowercase();
                let subject = commit.summary().unwrap_or("").to_lowercase();
                let author = commit.author().name().unwrap_or("").to_lowercase();
                let hash = id.to_string().to_lowercase();
                if !subject.contains(&q_l) && !author.contains(&q_l) && !hash.starts_with(&q_l) {
                    matches = false;
                }
            }

            if let Some(ref m) = filter.message {
                if !commit
                    .summary()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&m.to_lowercase())
                {
                    matches = false;
                }
            }

            if let Some(ref a) = filter.author {
                if !commit
                    .author()
                    .name()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&a.to_lowercase())
                {
                    matches = false;
                }
            }

            let is_merge = commit.parent_count() > 1;
            if filter.only_merges && !is_merge {
                matches = false;
            }
            if filter.no_merges && is_merge {
                matches = false;
            }

            if matches {
                matched_oids.push(id);
                matched_set.insert(id);
                if matched_oids.len() >= filter.limit {
                    break;
                }
            }
        }

        if matched_oids.is_empty() {
            return Ok(Vec::new());
        }

        // 2. Convert and simplify
        let mut final_results = Vec::new();
        for oid in matched_oids {
            let commit = self.repo.find_commit(oid)?;
            let mut info = self.convert_commit(&commit)?;

            let mut simplified_parents = Vec::new();
            for p_oid in commit.parent_ids() {
                if let Some(ancestor) = self.find_visible_ancestor(p_oid, &matched_set) {
                    if !simplified_parents.contains(&ancestor) {
                        simplified_parents.push(ancestor);
                    }
                }
            }
            info.parents = simplified_parents;
            final_results.push(info);
        }

        Ok(final_results)
    }

    fn find_visible_ancestor(
        &self,
        start_oid: Oid,
        matched_set: &std::collections::HashSet<Oid>,
    ) -> Option<String> {
        if matched_set.contains(&start_oid) {
            return Some(start_oid.to_string());
        }

        let mut walk = self.repo.revwalk().ok()?;
        let _ = walk.set_sorting(Sort::TOPOLOGICAL);
        walk.push(start_oid).ok()?;

        for id in walk {
            if let Ok(id) = id {
                if matched_set.contains(&id) {
                    return Some(id.to_string());
                }
            }
        }
        None
    }

    pub fn resolve_ref(&self, reference: &str) -> Result<String> {
        let obj = self.repo.revparse_single(reference)?;
        let commit = obj.peel_to_commit()?;
        Ok(commit.id().to_string())
    }

    pub fn compare(&self, branch1: &str, branch2: &str) -> Result<ComparisonResult> {
        let obj1 = self.repo.revparse_single(branch1)?;
        let obj2 = self.repo.revparse_single(branch2)?;

        let c1 = obj1
            .as_commit()
            .ok_or_else(|| anyhow!("Ref {} is not a commit", branch1))?;
        let c2 = obj2
            .as_commit()
            .ok_or_else(|| anyhow!("Ref {} is not a commit", branch2))?;

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
                let name_str = std::str::from_utf8(name)
                    .unwrap_or("")
                    .replace("refs/tags/", "");
                labels.push(LabelInfo::Tag(name_str));
            }
            true
        })?;

        Ok(labels)
    }
}
