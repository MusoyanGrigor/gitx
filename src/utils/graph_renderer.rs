use crate::models::{CommitInfo, LabelInfo};
use crate::utils::styles::{TreeStyle, styled};
use crossterm::style::Stylize;

/// Represents a lane state: a commit hash plus its assigned color index
#[derive(Clone, Debug)]
struct LaneState {
    hash: String,
    color_idx: usize,
}

pub struct GraphRenderer {
    active_lanes: Vec<Option<LaneState>>,
    next_color_idx: usize,
}

impl GraphRenderer {
    pub fn new() -> Self {
        Self { active_lanes: Vec::new(), next_color_idx: 0 }
    }

    pub fn render(&mut self, commits: &[CommitInfo]) {
        if commits.is_empty() {
            println!("{}", "No commits found.".dark_grey());
            return;
        }

        // Align metadata columns by calculating max lanes first
        let max_lanes = self.calculate_max_lanes(commits);
        let graph_area_width = (max_lanes * 3).max(6);

        println!(); // Spacing

        for (idx, commit) in commits.iter().enumerate() {
            let is_last = idx == commits.len() - 1;
            
            // 1. Where is our current commit? Assign or get lane + its color
            let node_lane_idx = self.get_or_assign_lane(&commit.hash);
            let node_lane_color = self.active_lanes[node_lane_idx].as_ref().unwrap().color_idx;
            
            // 2. Prepare next state (calculate lanes for children)
            let mut next_lanes = self.active_lanes.clone();
            next_lanes[node_lane_idx] = None; // Node is consumed
            
            // Add parents
            for (i, p_hash) in commit.parents.iter().enumerate() {
                // If parent is not tracked, add it to lanes
                if !next_lanes.iter().any(|l| l.as_ref().map_or(false, |ls| &ls.hash == p_hash)) {
                    let color = if i == 0 { node_lane_color } else { self.next_color() };
                    let state = LaneState { hash: p_hash.clone(), color_idx: color };
                    
                    let pos = next_lanes.iter().position(|l| l.is_none()).unwrap_or(next_lanes.len());
                    if pos == next_lanes.len() { next_lanes.push(Some(state)); } 
                    else { next_lanes[pos] = Some(state); }
                }
            }

            // 3. Render Node Row
            self.render_node_row(commit, node_lane_idx, graph_area_width);

            // 4. Render Connector (Transition) Row
            if !is_last {
                self.render_connector_row(&next_lanes, node_lane_idx, &commit.parents);
            }
            
            // 5. Commit state for next iteration
            self.active_lanes = next_lanes;
            self.trim_lanes();
        }
        
        println!(); // Spacing
    }

    fn calculate_max_lanes(&self, commits: &[CommitInfo]) -> usize {
        let mut sim_lanes: Vec<Option<String>> = self.active_lanes.iter().map(|l| l.as_ref().map(|s| s.hash.clone())).collect();
        let mut max = sim_lanes.len();
        for commit in commits {
            let pos = if let Some(p) = sim_lanes.iter().position(|l| l.as_ref() == Some(&commit.hash)) { p } 
                      else {
                           let p = sim_lanes.iter().position(|l| l.is_none()).unwrap_or(sim_lanes.len());
                           if p == sim_lanes.len() { sim_lanes.push(Some(commit.hash.clone())); } 
                           else { sim_lanes[p] = Some(commit.hash.clone()); }
                           p
                      };
            sim_lanes[pos] = None;
            for p in &commit.parents {
                if !sim_lanes.contains(&Some(p.clone())) {
                     let p_pos = sim_lanes.iter().position(|l| l.is_none()).unwrap_or(sim_lanes.len());
                     if p_pos == sim_lanes.len() { sim_lanes.push(Some(p.clone())); } else { sim_lanes[p_pos] = Some(p.clone()); }
                }
            }
            max = max.max(sim_lanes.len());
            while sim_lanes.last().map_or(false, |l| l.is_none()) { sim_lanes.pop(); }
        }
        max
    }

    fn next_color(&mut self) -> usize {
        let c = self.next_color_idx;
        self.next_color_idx += 1;
        c
    }

    fn get_or_assign_lane(&mut self, hash: &str) -> usize {
        if let Some(pos) = self.active_lanes.iter().position(|l| l.as_ref().map_or(false, |ls| &ls.hash == hash)) {
            pos
        } else {
            let color = self.next_color();
            let pos = self.active_lanes.iter().position(|l| l.is_none()).unwrap_or(self.active_lanes.len());
            let state = LaneState { hash: hash.to_string(), color_idx: color };
            if pos == self.active_lanes.len() { self.active_lanes.push(Some(state)); } 
            else { self.active_lanes[pos] = Some(state); }
            pos
        }
    }

    fn render_node_row(&self, commit: &CommitInfo, node_lane_idx: usize, graph_area_width: usize) {
        let is_head = commit.labels.iter().any(|l| matches!(l, LabelInfo::Head(_)));
        let is_merge = commit.parents.len() > 1;
        
        let lane_color = self.active_lanes[node_lane_idx].as_ref().unwrap().color_idx;
        
        let node_style = if is_head { TreeStyle::head_node() } 
                         else if is_merge { TreeStyle::merge_node(lane_color) }
                         else { TreeStyle::commit_node(lane_color) };
        let node_char = if is_head { "◉" } else if is_merge { "◎" } else { "●" };
        
        let mut graph = String::new();
        for (i, lane) in self.active_lanes.iter().enumerate() {
            if i == node_lane_idx {
                graph.push_str(&styled(node_char, node_style));
            } else if let Some(ls) = lane {
                graph.push_str(&styled("│", TreeStyle::connector(ls.color_idx)));
            } else {
                graph.push(' ');
            }
            graph.push_str("  ");
        }
        
        let prefix_len = self.active_lanes.len() * 3;
        if prefix_len < graph_area_width {
            for _ in 0..(graph_area_width - prefix_len) { graph.push(' '); }
        }

        let hash_short = if commit.hash.len() > 7 { &commit.hash[..7] } else { &commit.hash };
        let date_str = self.format_date(commit.date);
        let refs_str = self.format_labels_ide(&commit.labels);
        
        println!("{} {}  {:<35}  {} {} {} {}", 
            graph, 
            styled(hash_short, TreeStyle::hash()), 
            styled(&commit.subject, TreeStyle::subject()),
            styled(&commit.author, TreeStyle::metadata()), 
            styled("•", TreeStyle::separator()),
            styled(date_str, TreeStyle::metadata()),
            refs_str
        );
    }

    fn render_connector_row(&self, next_lanes: &[Option<LaneState>], node_lane_idx: usize, parents: &[String]) {
        if parents.is_empty() {
             // Root: continue other lanes
             if next_lanes.iter().any(|l| l.is_some()) {
                 let mut graph = String::new();
                 for lane in next_lanes.iter() {
                     if let Some(ls) = lane { graph.push_str(&styled("│", TreeStyle::connector(ls.color_idx))); }
                     else { graph.push(' '); }
                     graph.push_str("  ");
                 }
                 println!("{}", graph);
             }
             return;
        }

        let mut graph = String::new();
        let current_color = self.active_lanes[node_lane_idx].as_ref().unwrap().color_idx;
        let is_merge = parents.len() > 1;

        for (i, _) in self.active_lanes.iter().enumerate() {
            if i == node_lane_idx {
                 if is_merge {
                      graph.push_str(&styled("├─╮", TreeStyle::connector(current_color)));
                 } else if next_lanes.get(i).map_or(false, |l| l.is_some()) {
                      graph.push_str(&styled("│  ", TreeStyle::connector(current_color)));
                 } else {
                      graph.push_str("   ");
                 }
            } else {
                let curr = &self.active_lanes[i];
                let next = next_lanes.get(i).unwrap_or(&None);
                
                if let (Some(c_ls), Some(_n_ls)) = (curr, next) {
                    graph.push_str(&styled("│  ", TreeStyle::connector(c_ls.color_idx)));
                } else if let Some(c_ls) = curr {
                    // Lane ending or converging
                    graph.push_str(&styled("╰─╮", TreeStyle::connector(c_ls.color_idx)));
                } else if let Some(n_ls) = next {
                    // New lane starting or diverging
                    graph.push_str(&styled("  │", TreeStyle::connector(n_ls.color_idx)));
                } else {
                    graph.push_str("   ");
                }
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

    fn format_labels_ide(&self, labels: &[LabelInfo]) -> String {
        if labels.is_empty() { return String::new(); }
        let mut parts = Vec::new();
        let mut head_ref = None;
        for l in labels { if let LabelInfo::Head(n) = l { head_ref = Some(n.clone()); } }
        for l in labels {
            match l {
                LabelInfo::Head(_) => {}, 
                LabelInfo::LocalBranch(n) => {
                    if head_ref.as_ref() == Some(n) { parts.push(styled(format!("HEAD → {}", n), TreeStyle::head_badge())); } 
                    else { parts.push(styled(n, TreeStyle::local_branch_badge())); }
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
    fn test_colorful_history() {
        let mut renderer = GraphRenderer::new();
        let commits = vec![
            mock_commit("H", vec!["G"]),         
            mock_commit("G", vec!["E", "F"]),    
            mock_commit("F", vec!["D"]),         
            mock_commit("E", vec!["D"]),         
            mock_commit("D", vec!["C"]),         
            mock_commit("C", vec!["B"]),         
            mock_commit("B", vec!["A"]),         
            mock_commit("A", vec![]),            
        ];
        renderer.render(&commits);
    }
}
