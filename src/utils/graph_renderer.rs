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

        // Pre-scan to find max lanes for alignment
        let max_lanes = self.calculate_max_lanes(commits);
        let graph_width = (max_lanes * 3).max(6); // At least some space

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
                let pos = next_lanes.iter().position(|l| l.is_none()).unwrap_or(next_lanes.len());
                if pos == next_lanes.len() { next_lanes.push(Some(p)); } 
                else { next_lanes[pos] = Some(p); }
            }

            // Render COMPACT One-Line layout
            self.render_compact_row(commit, node_lane, graph_width);

            // Connectors (subtle)
            if !is_last {
                self.render_connector_row(&next_lanes, node_lane, &commit.parents);
            }
            
            self.active_lanes = next_lanes;
            self.trim_lanes();
        }
        
        println!(); // Spacing
    }

    fn calculate_max_lanes(&self, commits: &[CommitInfo]) -> usize {
        // Simple heuristic for pre-scan
        let mut temp_lanes: Vec<Option<String>> = self.active_lanes.clone();
        let mut max = temp_lanes.len();
        
        for commit in commits {
            // This is a rough simulation
            let node_lane = if let Some(pos) = temp_lanes.iter().position(|l| l.as_ref() == Some(&commit.hash)) {
                pos
            } else {
                let pos = temp_lanes.iter().position(|l| l.is_none()).unwrap_or(temp_lanes.len());
                if pos == temp_lanes.len() { temp_lanes.push(Some(commit.hash.clone())); } 
                else { temp_lanes[pos] = Some(commit.hash.clone()); }
                pos
            };
            
            temp_lanes[node_lane] = None;
            for (i, p) in commit.parents.iter().enumerate() {
                if !temp_lanes.iter().any(|l| l.as_ref() == Some(p)) {
                    if i == 0 { temp_lanes[node_lane] = Some(p.clone()); } 
                    else {
                        let pos = temp_lanes.iter().position(|l| l.is_none()).unwrap_or(temp_lanes.len());
                        if pos == temp_lanes.len() { temp_lanes.push(Some(p.clone())); } 
                        else { temp_lanes[pos] = Some(p.clone()); }
                    }
                }
            }
            max = max.max(temp_lanes.len());
            while temp_lanes.last().map_or(false, |l| l.is_none()) { temp_lanes.pop(); }
        }
        max
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

    fn render_compact_row(&self, commit: &CommitInfo, node_lane: usize, graph_width: usize) {
        let is_head = commit.labels.iter().any(|l| matches!(l, LabelInfo::Head(_)));
        let is_merge = commit.parents.len() > 1;
        
        let node_style = if is_head { TreeStyle::head_node() } 
                         else if is_merge { TreeStyle::merge_node() }
                         else { TreeStyle::commit_node() };
        
        // 1. Build graph prefix
        let mut prefix = String::new();
        for (i, lane) in self.active_lanes.iter().enumerate() {
            if i == node_lane {
                prefix.push_str(&styled("●", node_style));
            } else if lane.is_some() {
                prefix.push_str(&styled("│", TreeStyle::connector()));
            } else {
                prefix.push(' ');
            }
            prefix.push_str("  ");
        }
        
        // Pad graph area
        let prefix_len = self.active_lanes.len() * 3;
        if prefix_len < graph_width {
            for _ in 0..(graph_width - prefix_len) { prefix.push(' '); }
        }

        // 2. Format Hash & Subject
        let hash_short = if commit.hash.len() > 7 { &commit.hash[..7] } else { &commit.hash };
        let subject = &commit.subject;
        
        // 3. Metadata & Refs
        let date_str = self.format_date(commit.date);
        let refs_str = self.format_labels_compact(&commit.labels);
        
        // 4. Combined output
        println!("{} {}  {:<40}  {} {} {} {}", 
            prefix, 
            styled(hash_short, TreeStyle::hash()), 
            styled(subject, TreeStyle::subject()),
            styled(&commit.author, TreeStyle::metadata()), 
            styled("•", TreeStyle::separator()),
            styled(date_str, TreeStyle::metadata()),
            refs_str
        );
    }

    fn render_connector_row(&self, next_lanes: &[Option<String>], node_lane: usize, parents: &[String]) {
        let style = TreeStyle::connector();
        if parents.is_empty() {
            if next_lanes.iter().any(|l| l.is_some()) {
                let mut graph = String::new();
                for lane in next_lanes.iter() {
                    if lane.is_some() { graph.push_str(&styled("│", style)); } 
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
                if is_merge { graph.push_str(&styled("├─╮", style)); } 
                else if next_lanes.get(i).map_or(false, |l| l.is_some()) { graph.push_str(&styled("│  ", style)); } 
                else { graph.push_str("   "); }
            } else {
                let current_exists = self.active_lanes[i].is_some();
                let next_exists = next_lanes.get(i).map_or(false, |l| l.is_some());
                if current_exists && next_exists { graph.push_str(&styled("│  ", style)); } 
                else if current_exists && !next_exists { graph.push_str(&styled("╰─╮", style)); } 
                else if !current_exists && next_exists { graph.push_str(&styled("  │", style)); } 
                else { graph.push_str("   "); }
            }
        }
        println!("{}", graph);
    }

    fn trim_lanes(&mut self) {
        while self.active_lanes.last().map_or(false, |l| l.is_none()) { self.active_lanes.pop(); }
    }

    fn format_date(&self, timestamp: i64) -> String {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        let diff = now.saturating_sub(timestamp as u64);
        if diff < 60 { "now".into() }
        else if diff < 3600 { format!("{}m", diff / 60) }
        else if diff < 86400 { format!("{}h", diff / 3600) }
        else if diff < 2592000 { format!("{}d", diff / 86400) }
        else { format!("{}mo", diff / 2592000) }
    }

    fn format_labels_compact(&self, labels: &[LabelInfo]) -> String {
        if labels.is_empty() { return String::new(); }
        let mut parts = Vec::new();
        let mut head_ref = None;
        for l in labels { if let LabelInfo::Head(n) = l { head_ref = Some(n.clone()); } }

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
                LabelInfo::RemoteBranch(n) => { parts.push(styled(format!("origin/{}", n), TreeStyle::remote_branch_badge())); },
                LabelInfo::Tag(n) => { parts.push(styled(format!("tag:{}", n), TreeStyle::tag_badge())); },
            }
        }
        if parts.is_empty() && head_ref.is_some() { parts.push(styled("HEAD", TreeStyle::head_badge())); }
        if parts.is_empty() { return String::new(); }
        format!("{} {}", styled("•", TreeStyle::separator()), parts.join(&styled(" | ", TreeStyle::ref_divider())))
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
    fn test_compact_history() {
        let mut renderer = GraphRenderer::new();
        let commits = vec![
            mock_commit("C", vec!["B"]),
            mock_commit("B", vec!["A"]),
            mock_commit("A", vec![]),
        ];
        renderer.render(&commits);
    }
}
