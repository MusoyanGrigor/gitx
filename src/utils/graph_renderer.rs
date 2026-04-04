use crate::models::{CommitInfo, LabelInfo};
use crate::utils::styles::{TreeStyle, styled};
use crossterm::style::Stylize;

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
        Self { active_lanes: Vec::new(), next_color_idx: 1 } // Start side colors from 1
    }

    pub fn render(&mut self, commits: &[CommitInfo]) {
        if commits.is_empty() {
            println!("{}", "No commits found.".dark_grey());
            return;
        }

        let max_lanes = 4; // Compact enough for this style
        let graph_area_width = max_lanes * 3;

        println!(); 

        for (idx, commit) in commits.iter().enumerate() {
            let is_last = idx == commits.len() - 1;
            
            // 1. Assign/get lane
            let node_lane_idx = self.get_or_assign_lane(&commit.hash);
            let lane_color = self.active_lanes[node_lane_idx].as_ref().unwrap().color_idx;
            
            // 2. Prepare next state
            let mut next_lanes = self.active_lanes.clone();
            next_lanes[node_lane_idx] = None;
            
            for (i, p_hash) in commit.parents.iter().enumerate() {
                if !next_lanes.iter().any(|l| l.as_ref().map_or(false, |ls| &ls.hash == p_hash)) {
                    // Force parent 0 into trunk if possible, or same lane
                    let color = if node_lane_idx == 0 { 0 } else if i == 0 { lane_color } else { self.next_color() };
                    let state = LaneState { hash: p_hash.clone(), color_idx: color };
                    
                    let pos = if i == 0 && node_lane_idx == 0 { 0 } 
                              else { next_lanes.iter().position(|l| l.is_none()).unwrap_or(next_lanes.len()) };
                    
                    if pos >= next_lanes.len() { next_lanes.push(Some(state)); } 
                    else { next_lanes[pos] = Some(state); }
                }
            }

            // 3. Render Connector ABOVE (Arc start)
            if node_lane_idx > 0 {
                self.render_arc_start(node_lane_idx);
            }

            // 4. Render Node Row
            self.render_ide_row(commit, node_lane_idx, graph_area_width);

            // 5. Render Connector BELOW (Arc end)
            if !is_last {
                 if node_lane_idx > 0 {
                    self.render_arc_end(node_lane_idx);
                 } else {
                    self.render_main_trunk_connector(&next_lanes);
                 }
            }
            
            self.active_lanes = next_lanes;
            self.trim_lanes();
        }
        
        println!(); 
    }

    fn next_color(&mut self) -> usize {
        let c = self.next_color_idx;
        self.next_color_idx = (self.next_color_idx + 1) % 6;
        if self.next_color_idx == 0 { self.next_color_idx = 1; } // Skip lane 0 blue
        c
    }

    fn get_or_assign_lane(&mut self, hash: &str) -> usize {
        if let Some(pos) = self.active_lanes.iter().position(|l| l.as_ref().map_or(false, |ls| &ls.hash == hash)) {
            pos
        } else {
            // First commit usually goes to trunk
            let pos = if self.active_lanes.is_empty() { 0 } else { self.active_lanes.iter().position(|l| l.is_none()).unwrap_or(self.active_lanes.len()) };
            let color = if pos == 0 { 0 } else { self.next_color() };
            let state = LaneState { hash: hash.to_string(), color_idx: color };
            if pos >= self.active_lanes.len() { self.active_lanes.push(Some(state)); } 
            else { self.active_lanes[pos] = Some(state); }
            pos
        }
    }

    fn render_ide_row(&self, commit: &CommitInfo, node_lane_idx: usize, graph_area_width: usize) {
        let _is_merge = commit.parents.len() > 1;
        let is_head = commit.labels.iter().any(|l| matches!(l, LabelInfo::Head(_)));
        
        let lane_color = self.active_lanes[node_lane_idx].as_ref().unwrap().color_idx;
        let node_style = if is_head { TreeStyle::head_node() } 
                         else if node_lane_idx == 0 { TreeStyle::merge_node(0) }
                         else { TreeStyle::commit_node(lane_color) };
        
        // Icon: hollow for main line merges/anchors, solid for side commits
        let node_char = if is_head { "◉" } else if node_lane_idx == 0 { "○" } else { "●" };
        
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

        let _hash_short = if commit.hash.len() > 7 { &commit.hash[..7] } else { &commit.hash };
        let refs_str = self.format_labels_badge(&commit.labels);
        
        println!("{}  {:<45}  {}", 
            graph, 
            styled(&commit.subject, TreeStyle::subject()),
            refs_str
        );
    }

    fn render_arc_start(&self, lane_idx: usize) {
        let trunk_style = TreeStyle::connector(0);
        let branch_color = self.active_lanes[lane_idx].as_ref().unwrap().color_idx;
        let mut g = String::new();
        g.push_str(&styled("│", trunk_style));
        g.push_str(&styled("╭", TreeStyle::connector(branch_color)));
        for i in 1..lane_idx {
            if self.active_lanes[i].is_some() {
                 g.push_str(&styled("│", TreeStyle::connector(self.active_lanes[i].as_ref().unwrap().color_idx)));
            } else { g.push_str("─"); }
        }
        println!("{}", g);
    }

    fn render_arc_end(&self, lane_idx: usize) {
        let trunk_style = TreeStyle::connector(0);
        let branch_color = self.active_lanes[lane_idx].as_ref().unwrap().color_idx;
        let mut g = String::new();
        g.push_str(&styled("│", trunk_style));
        g.push_str(&styled("╰", TreeStyle::connector(branch_color)));
        for i in 1..lane_idx {
             if self.active_lanes[i].is_some() {
                 g.push_str(&styled("│", TreeStyle::connector(self.active_lanes[i].as_ref().unwrap().color_idx)));
            } else { g.push_str("─"); }
        }
        println!("{}", g);
    }

    fn render_main_trunk_connector(&self, next_lanes: &[Option<LaneState>]) {
        let mut g = String::new();
        for (_i, lane) in next_lanes.iter().enumerate() {
            if let Some(ls) = lane { g.push_str(&styled("│", TreeStyle::connector(ls.color_idx))); }
            else { g.push(' '); }
            g.push_str("  ");
        }
        println!("{}", g);
    }

    fn trim_lanes(&mut self) {
        while self.active_lanes.last().map_or(false, |l| l.is_none()) { self.active_lanes.pop(); }
    }

    fn format_labels_badge(&self, labels: &[LabelInfo]) -> String {
        if labels.is_empty() { return String::new(); }
        let mut head_ref = None;
        for l in labels { if let LabelInfo::Head(n) = l { head_ref = Some(n.clone()); } }
        
        let mut badges = Vec::new();
        for l in labels {
            match l {
                LabelInfo::Head(_) => {}, 
                LabelInfo::LocalBranch(n) => {
                    let text = if head_ref.as_ref() == Some(n) { format!("◎ {}", n) } else { n.clone() };
                    badges.push(styled(format!(" {} ", text), TreeStyle::local_branch_badge().on_blue().black()));
                },
                LabelInfo::RemoteBranch(n) => {
                    badges.push(styled(format!(" origin/{} ", n), TreeStyle::remote_branch_badge().on_black()));
                },
                LabelInfo::Tag(n) => {
                    badges.push(styled(format!(" {} ", n), TreeStyle::tag_badge().on_yellow().black()));
                },
            }
        }
        if badges.is_empty() && head_ref.is_some() {
            badges.push(styled(" HEAD ", TreeStyle::head_badge().on_cyan().black()));
        }
        badges.join(" ")
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
    fn test_ide_arcs() {
        let mut renderer = GraphRenderer::new();
        let commits = vec![
            mock_commit("H", vec!["G"]),         
            mock_commit("G", vec!["F", "E"]),    
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
