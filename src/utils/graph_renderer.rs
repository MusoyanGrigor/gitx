use crate::models::{CommitInfo, LabelInfo};
use crate::utils::styles::{TreeStyle, styled};
use crossterm::style::Stylize;

pub struct GraphRenderer {
    active_lanes: Vec<Option<String>>,
}

impl GraphRenderer {
    pub fn new() -> Self {
        Self { active_lanes: Vec::new() }
    }

    pub fn render(&mut self, commits: &[CommitInfo]) {
        if commits.is_empty() {
            println!("{}", "No commits found.".dark_grey());
            return;
        }

        println!(); // Spacing

        for (idx, commit) in commits.iter().enumerate() {
            let is_last = idx == commits.len() - 1;
            let node_lane = self.get_or_assign_lane(&commit.hash);
            
            let mut next_lanes = self.active_lanes.clone();
            next_lanes[node_lane] = None;
            
            let mut added_parents = Vec::new();
            for (i, parent) in commit.parents.iter().enumerate() {
                if !next_lanes.iter().any(|l| l.as_ref() == Some(parent)) {
                    if i == 0 {
                        next_lanes[node_lane] = Some(parent.clone());
                    } else {
                        added_parents.push(parent.clone());
                    }
                }
            }
            for p in added_parents {
                if let Some(empty_idx) = next_lanes.iter().position(|l| l.is_none()) {
                    next_lanes[empty_idx] = Some(p);
                } else {
                    next_lanes.push(Some(p));
                }
            }

            // 3. Main Commit Line
            self.render_node_row(commit, node_lane);
            
            // 4. Metadata Line
            self.render_metadata_row(commit, node_lane, &next_lanes);

            // 5. Connectors
            if !is_last {
                self.render_connector_row(&next_lanes, node_lane, &commit.parents);
            }
            
            self.active_lanes = next_lanes;
            self.trim_lanes();
        }
        
        println!(); // Spacing
    }

    fn get_or_assign_lane(&mut self, hash: &str) -> usize {
        if let Some(pos) = self.active_lanes.iter().position(|l| l.as_ref() == Some(&hash.to_string())) {
            pos
        } else {
            let pos = self.active_lanes.iter().position(|l| l.is_none()).unwrap_or(self.active_lanes.len());
            if pos == self.active_lanes.len() {
                self.active_lanes.push(Some(hash.to_string()));
            } else {
                self.active_lanes[pos] = Some(hash.to_string());
            }
            pos
        }
    }

    fn render_node_row(&self, commit: &CommitInfo, node_lane: usize) {
        let is_head = commit.labels.iter().any(|l| matches!(l, LabelInfo::Head(_)));
        let node_style = if is_head { TreeStyle::head_node() } else { TreeStyle::commit_node() };
        let node_char = if is_head { "●" } else { "●" }; // Same char but different style
        
        let mut graph = String::new();
        for (i, lane) in self.active_lanes.iter().enumerate() {
            if i == node_lane {
                graph.push_str(&styled(node_char, node_style));
            } else if lane.is_some() {
                graph.push_str(&styled("│", TreeStyle::connector()));
            } else {
                graph.push(' ');
            }
            graph.push_str("  ");
        }
        
        let hash_short = if commit.hash.len() > 7 { &commit.hash[..7] } else { &commit.hash };
        
        println!("{} {}  {}", 
            graph, 
            styled(hash_short, TreeStyle::hash()), 
            styled(&commit.subject, TreeStyle::subject())
        );
    }

    fn render_metadata_row(&self, commit: &CommitInfo, node_lane: usize, next_lanes: &[Option<String>]) {
        let mut graph = String::new();
        for (i, lane) in self.active_lanes.iter().enumerate() {
            let next_exists = next_lanes.get(i).map_or(false, |l| l.is_some());
            if i == node_lane && next_exists {
                graph.push_str(&styled("│", TreeStyle::connector()));
            } else if lane.is_some() && next_exists {
                graph.push_str(&styled("│", TreeStyle::connector()));
            } else {
                graph.push(' ');
            }
            graph.push_str("  ");
        }

        let date_str = self.format_date(commit.date);
        let refs_str = self.format_labels_ide(&commit.labels);
        
        print!("{} {} {} {}", 
            graph, 
            styled(&commit.author, TreeStyle::metadata()), 
            styled("•", TreeStyle::separator()),
            styled(date_str, TreeStyle::metadata())
        );
        
        if !refs_str.is_empty() {
             print!(" {} {}", styled("•", TreeStyle::separator()), refs_str);
        }
        println!();
    }

    fn render_connector_row(&self, next_lanes: &[Option<String>], node_lane: usize, parents: &[String]) {
        let connector_style = TreeStyle::connector();
        
        if parents.is_empty() {
            if next_lanes.iter().any(|l| l.is_some()) {
                let mut graph = String::new();
                for lane in next_lanes.iter() {
                    if lane.is_some() { graph.push_str(&styled("│", connector_style)); } 
                    else { graph.push(' '); }
                    graph.push_str("  ");
                }
                println!("{}", graph);
            }
            return;
        }

        let mut graph = String::new();
        let is_merge = parents.len() > 1;

        for (i, _) in self.active_lanes.iter().enumerate() {
            if i == node_lane {
                if is_merge {
                    graph.push_str(&styled("├─╮", connector_style));
                } else if next_lanes.get(i).map_or(false, |l| l.is_some()) {
                    graph.push_str(&styled("│  ", connector_style));
                } else {
                    graph.push_str("   ");
                }
            } else {
                let current_exists = self.active_lanes[i].is_some();
                let next_exists = next_lanes.get(i).map_or(false, |l| l.is_some());
                
                if current_exists && next_exists {
                    graph.push_str(&styled("│  ", connector_style));
                } else if current_exists && !next_exists {
                    graph.push_str(&styled("╰─╮", connector_style));
                } else if !current_exists && next_exists {
                    graph.push_str(&styled("  │", connector_style));
                } else {
                    graph.push_str("   ");
                }
            }
        }
        println!("{}", graph);
    }

    fn trim_lanes(&mut self) {
        while self.active_lanes.last().map_or(false, |l| l.is_none()) {
            self.active_lanes.pop();
        }
    }

    fn format_date(&self, timestamp: i64) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let diff = now.saturating_sub(timestamp as u64);
        
        if diff < 60 { "just now".into() }
        else if diff < 3600 { format!("{}m ago", diff / 60) }
        else if diff < 86400 { format!("{}h ago", diff / 3600) }
        else if diff < 2592000 { format!("{} days ago", diff / 86400) }
        else { format!("{} months ago", diff / 2592000) }
    }

    fn format_labels_ide(&self, labels: &[LabelInfo]) -> String {
        if labels.is_empty() { return String::new(); }
        let mut parts = Vec::new();
        
        let mut head_ref = None;
        for l in labels {
            if let LabelInfo::Head(n) = l { head_ref = Some(n.clone()); }
        }

        for l in labels {
            match l {
                LabelInfo::Head(_) => {}, 
                LabelInfo::LocalBranch(n) => {
                    if head_ref.as_ref() == Some(n) {
                        parts.push(styled(format!("HEAD → {}", n), TreeStyle::head_badge()));
                    } else {
                        parts.push(styled(n, TreeStyle::local_branch_badge()));
                    }
                },
                LabelInfo::RemoteBranch(n) => {
                    parts.push(styled(format!("origin/{}", n), TreeStyle::remote_branch_badge()));
                },
                LabelInfo::Tag(n) => {
                    parts.push(styled(format!("tag:{}", n), TreeStyle::tag_badge()));
                },
            }
        }
        
        if parts.is_empty() && head_ref.is_some() {
             // Only HEAD ref exists (detached or head on hash)
             parts.push(styled("HEAD", TreeStyle::head_badge()));
        }

        if parts.is_empty() { return String::new(); }
        parts.join(&styled(" | ", TreeStyle::ref_divider()))
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
    fn test_styled_history() {
        let mut renderer = GraphRenderer::new();
        let commits = vec![
            mock_commit("C", vec!["B"]),
            mock_commit("B", vec!["A"]),
            mock_commit("A", vec![]),
        ];
        renderer.render(&commits);
    }
}
