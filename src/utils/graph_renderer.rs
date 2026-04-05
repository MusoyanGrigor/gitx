use crate::models::{CommitInfo, LabelInfo};
use crate::utils::styles::{styled, TreeStyle};
use crossterm::style::Stylize;
use std::collections::HashSet;

pub struct GraphRenderer {
    active_lanes: Vec<Option<String>>, // Stores hashes of parents we are waiting for
    visible_hashes: HashSet<String>,
    pub use_ascii: bool,
}

impl GraphRenderer {
    pub fn new() -> Self {
        Self {
            active_lanes: Vec::new(),
            visible_hashes: HashSet::new(),
            use_ascii: false,
        }
    }

    pub fn render(&mut self, commits: &[CommitInfo]) {
        if commits.is_empty() {
            println!("{}", "No commits found.".dark_grey());
            return;
        }

        // Pre-scan to know what's visible
        self.visible_hashes = commits.iter().map(|c| c.hash.clone()).collect();
        let max_lanes = self.calculate_max_lanes(commits);
        let graph_width = (max_lanes * 2).max(4);

        println!(); // Spacing before tree

        for (idx, commit) in commits.iter().enumerate() {
            // 1. Identify which lane our current commit belongs to
            let node_lane_idx = self.get_or_assign_lane(&commit.hash);

            // 2. Render the node row
            self.render_node_row(commit, node_lane_idx, graph_width);

            // 3. Render a connector row reflecting the transition
            if idx < commits.len() - 1 {
                self.render_transition(commit, node_lane_idx);
            }

            // 4. Update lanes for next iteration
            self.update_lanes(commit, node_lane_idx);
        }

        println!(); // Spacing after tree
    }

    fn render_transition(&self, commit: &CommitInfo, node_lane_idx: usize) {
        let mut graph = String::new();
        let conn_char = if self.use_ascii { "|" } else { "│" };
        let visible_parents: Vec<_> = commit
            .parents
            .iter()
            .filter(|p| self.visible_hashes.contains(*p))
            .collect();
        let is_split = visible_parents.len() > 1;

        for (i, lane) in self.active_lanes.iter().enumerate() {
            if i == node_lane_idx {
                if is_split {
                    graph.push_str(&styled(
                        if self.use_ascii { "|-" } else { "├─╮" },
                        TreeStyle::connector(i),
                    ));
                } else if !visible_parents.is_empty() {
                    graph.push_str(&styled(conn_char, TreeStyle::connector(i)));
                    graph.push(' ');
                } else {
                    graph.push_str("  ");
                }
            } else if let Some(_) = lane {
                graph.push_str(&styled(conn_char, TreeStyle::connector(i)));
                graph.push(' ');
            } else {
                graph.push_str("  ");
            }
        }

        if !graph.trim().is_empty() {
            println!("{}", graph);
        }
    }

    fn calculate_max_lanes(&self, commits: &[CommitInfo]) -> usize {
        let mut sim_lanes: Vec<Option<String>> = Vec::new();
        let mut max = 0;
        let visible = &self.visible_hashes;

        for commit in commits {
            let pos = if let Some(p) = sim_lanes
                .iter()
                .position(|l| l.as_ref() == Some(&commit.hash))
            {
                p
            } else {
                let p = sim_lanes
                    .iter()
                    .position(|l| l.is_none())
                    .unwrap_or(sim_lanes.len());
                if p == sim_lanes.len() {
                    sim_lanes.push(Some(commit.hash.clone()));
                } else {
                    sim_lanes[p] = Some(commit.hash.clone());
                }
                p
            };
            sim_lanes[pos] = None;
            for p in &commit.parents {
                if visible.contains(p) && !sim_lanes.iter().any(|l| l.as_ref() == Some(p)) {
                    let p_pos = sim_lanes
                        .iter()
                        .position(|l| l.is_none())
                        .unwrap_or(sim_lanes.len());
                    if p_pos == sim_lanes.len() {
                        sim_lanes.push(Some(p.clone()));
                    } else {
                        sim_lanes[p_pos] = Some(p.clone());
                    }
                }
            }
            max = max.max(sim_lanes.len());
            while sim_lanes.last().map_or(false, |l| l.is_none()) {
                sim_lanes.pop();
            }
        }
        max
    }

    fn get_or_assign_lane(&mut self, hash: &str) -> usize {
        if let Some(pos) = self
            .active_lanes
            .iter()
            .position(|l| l.as_ref() == Some(&hash.to_string()))
        {
            pos
        } else {
            let pos = self
                .active_lanes
                .iter()
                .position(|l| l.is_none())
                .unwrap_or(self.active_lanes.len());
            if pos == self.active_lanes.len() {
                self.active_lanes.push(Some(hash.to_string()));
            } else {
                self.active_lanes[pos] = Some(hash.to_string());
            }
            pos
        }
    }

    fn update_lanes(&mut self, commit: &CommitInfo, node_lane_idx: usize) {
        self.active_lanes[node_lane_idx] = None;
        for p in &commit.parents {
            if self.visible_hashes.contains(p)
                && !self.active_lanes.iter().any(|l| l.as_ref() == Some(p))
            {
                let pos = self
                    .active_lanes
                    .iter()
                    .position(|l| l.is_none())
                    .unwrap_or(self.active_lanes.len());
                if pos == self.active_lanes.len() {
                    self.active_lanes.push(Some(p.clone()));
                } else {
                    self.active_lanes[pos] = Some(p.clone());
                }
            }
        }
        while self.active_lanes.last().map_or(false, |l| l.is_none()) {
            self.active_lanes.pop();
        }
    }

    fn render_node_row(&self, commit: &CommitInfo, node_lane_idx: usize, graph_width: usize) {
        let is_head = commit
            .labels
            .iter()
            .any(|l| matches!(l, LabelInfo::Head(_)));
        let is_merge = commit.parents.len() > 1;

        let node_char = if self.use_ascii {
            if is_head {
                "@"
            } else if is_merge {
                "M"
            } else {
                "*"
            }
        } else {
            if is_head {
                "◉"
            } else if is_merge {
                "◎"
            } else {
                "●"
            }
        };
        let node_style = if is_head {
            TreeStyle::head_node()
        } else if is_merge {
            TreeStyle::merge_node(node_lane_idx)
        } else {
            TreeStyle::commit_node(node_lane_idx)
        };

        let mut graph = String::new();
        let conn_char = if self.use_ascii { "|" } else { "│" };

        for (i, lane) in self.active_lanes.iter().enumerate() {
            if i == node_lane_idx {
                graph.push_str(&styled(node_char, node_style));
            } else if let Some(_) = lane {
                graph.push_str(&styled(conn_char, TreeStyle::connector(i)));
            } else {
                graph.push(' ');
            }
            graph.push(' ');
        }

        let current_width = self.active_lanes.len() * 2;
        if current_width < graph_width {
            for _ in 0..(graph_width - current_width) {
                graph.push(' ');
            }
        }

        let hash_short = if commit.hash.len() > 7 {
            &commit.hash[..7]
        } else {
            &commit.hash
        };
        let date_str = self.format_date(commit.date);
        let refs_str = self.format_labels_ide(&commit.labels);

        println!(
            "{} {}  {:<40}  {} {} {} {}",
            graph,
            styled(hash_short, TreeStyle::hash()),
            styled(&commit.subject, TreeStyle::subject()),
            styled(&commit.author, TreeStyle::metadata()),
            styled("•", TreeStyle::separator()),
            styled(date_str, TreeStyle::metadata()),
            refs_str
        );
    }

    fn format_date(&self, timestamp: i64) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let diff = now.saturating_sub(timestamp as u64);
        if diff < 60 {
            "now".into()
        } else if diff < 3600 {
            format!("{}m", diff / 60)
        } else if diff < 86400 {
            format!("{}h", diff / 3600)
        } else if diff < 2592000 {
            format!("{}d", diff / 86400)
        } else {
            format!("{}mo", diff / 2592000)
        }
    }

    fn format_labels_ide(&self, labels: &[LabelInfo]) -> String {
        if labels.is_empty() {
            return String::new();
        }
        let mut parts = Vec::new();
        let mut head_ref = None;
        for l in labels {
            if let LabelInfo::Head(n) = l {
                head_ref = Some(n.clone());
            }
        }
        for l in labels {
            match l {
                LabelInfo::Head(_) => {}
                LabelInfo::LocalBranch(n) => {
                    if head_ref.as_ref() == Some(n) {
                        parts.push(styled(format!("HEAD → {}", n), TreeStyle::head_badge()));
                    } else {
                        parts.push(styled(n, TreeStyle::local_branch_badge()));
                    }
                }
                LabelInfo::RemoteBranch(n) => {
                    parts.push(styled(
                        format!("origin/{}", n),
                        TreeStyle::remote_branch_badge(),
                    ));
                }
                LabelInfo::Tag(n) => {
                    parts.push(styled(format!("tag:{}", n), TreeStyle::tag_badge()));
                }
            }
        }
        if parts.is_empty() && head_ref.is_some() {
            parts.push(styled("HEAD", TreeStyle::head_badge()));
        }
        if parts.is_empty() {
            return String::new();
        }
        format!(
            "{} {}",
            styled("•", TreeStyle::separator()),
            parts.join(&styled(" | ", TreeStyle::ref_divider()))
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_commit(hash: &str, parents: Vec<&str>) -> CommitInfo {
        CommitInfo {
            hash: hash.to_string(),
            author: "author".to_string(),
            date: 0,
            subject: format!("commit {}", hash),
            body: None,
            labels: vec![],
            parents: parents.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_tree_simple() {
        let mut renderer = GraphRenderer::new();
        let commits = vec![
            mock_commit("C", vec!["B"]),
            mock_commit("B", vec!["A"]),
            mock_commit("A", vec![]),
        ];
        renderer.render(&commits);
    }

    #[test]
    fn test_tree_filtered() {
        let mut renderer = GraphRenderer::new();
        // Suppose B was hidden by filter, but simplified parent connects C to A directly
        let commits = vec![mock_commit("C", vec!["A"]), mock_commit("A", vec![])];
        renderer.render(&commits);
    }
}
