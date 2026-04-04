use crate::models::{CommitInfo, LabelInfo};

pub struct GraphRenderer {
    active_lanes: Vec<Option<String>>,
}

impl GraphRenderer {
    pub fn new() -> Self {
        Self { active_lanes: Vec::new() }
    }

    pub fn render(&mut self, commits: &[CommitInfo]) {
        for commit in commits {
            // Find or assign lane for the current commit
            let current_lane = self.get_or_assign_lane(&commit.hash);
            
            // Build visual prefix representing the graph connectors
            let mut prefix = String::new();
            for (i, lane) in self.active_lanes.iter().enumerate() {
                if i == current_lane {
                    prefix.push('*');
                } else if lane.is_some() {
                    prefix.push('|');
                } else {
                    prefix.push(' ');
                }
                prefix.push(' ');
            }
            
            // Render labels
            let label_str = self.format_labels(&commit.labels);
            
            // Output the graph line
            println!("{:<12} {} {}{}", 
                prefix, 
                &commit.hash[..7], 
                commit.subject, 
                label_str
            );
            
            // Update lanes for subsequent commits
            self.update_lanes(commit, current_lane);
        }
    }

    fn get_or_assign_lane(&mut self, hash: &str) -> usize {
        if let Some(pos) = self.active_lanes.iter().position(|l| l.as_ref() == Some(&hash.to_string())) {
            pos
        } else {
            // Hash not found in existing lanes (e.g., new branch head or floating commit)
            // Try to find an empty slot or extend lanes
            let pos = self.active_lanes.iter().position(|l| l.is_none()).unwrap_or(self.active_lanes.len());
            if pos == self.active_lanes.len() {
                self.active_lanes.push(Some(hash.to_string()));
            } else {
                self.active_lanes[pos] = Some(hash.to_string());
            }
            pos
        }
    }

    fn update_lanes(&mut self, commit: &CommitInfo, current_lane: usize) {
        // Clear current commit hash from the lane
        self.active_lanes[current_lane] = None;
        
        // Add parents
        for (i, parent) in commit.parents.iter().enumerate() {
            // Don't add if already tracked in another lane
            if !self.active_lanes.iter().any(|l| l.as_ref() == Some(parent)) {
                if i == 0 {
                    // First parent: use current lane
                    self.active_lanes[current_lane] = Some(parent.clone());
                } else {
                    // Other parents: find a new lane slot
                    let pos = self.active_lanes.iter().position(|l| l.is_none()).unwrap_or(self.active_lanes.len());
                    if pos == self.active_lanes.len() {
                        self.active_lanes.push(Some(parent.clone()));
                    } else {
                        self.active_lanes[pos] = Some(parent.clone());
                    }
                }
            }
        }
        
        // Trim trailing empty lanes to keep layout compact
        while self.active_lanes.last().map_or(false, |l| l.is_none()) {
            self.active_lanes.pop();
        }
    }

    fn format_labels(&self, labels: &[LabelInfo]) -> String {
        if labels.is_empty() { return String::new(); }
        let s: Vec<String> = labels.iter().map(|l| match l {
            LabelInfo::Head(n) => format!("HEAD -> {}", n),
            LabelInfo::LocalBranch(n) => n.clone(),
            LabelInfo::RemoteBranch(n) => format!("origin/{}", n),
            LabelInfo::Tag(n) => format!("tag:{}", n),
        }).collect();
        format!(" \x1b[33m({})\x1b[0m", s.join(", ")) // Yellow text
    }
}
