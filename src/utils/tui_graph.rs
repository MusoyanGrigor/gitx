use crate::models::{CommitInfo, LabelInfo};
use ratatui::style::{Color, Style, Modifier};
use ratatui::text::{Span, Line};

#[derive(Clone, Debug)]
pub struct LaneState {
    pub hash: String,
    pub color_idx: usize,
}

pub struct TuiGraphRenderer {
    active_lanes: Vec<Option<LaneState>>,
    next_color_idx: usize,
}

impl TuiGraphRenderer {
    pub fn new() -> Self {
        Self { active_lanes: Vec::new(), next_color_idx: 1 }
    }

    pub fn get_lane_color(idx: usize) -> Color {
        match idx % 6 {
            0 => Color::Blue,
            1 => Color::Green,
            2 => Color::Yellow,
            3 => Color::Magenta,
            4 => Color::Cyan,
            _ => Color::Red,
        }
    }

    pub fn compute_rows(&mut self, commits: &[CommitInfo]) -> Vec<GraphRow> {
        let mut rows = Vec::new();
        for commit in commits {
            let node_lane_idx = self.get_or_assign_lane(&commit.hash);
            let color_idx = self.active_lanes[node_lane_idx].as_ref().unwrap().color_idx;

            // Prepare for next iteration
            let mut next_lanes = self.active_lanes.clone();
            next_lanes[node_lane_idx] = None;
            for (i, p) in commit.parents.iter().enumerate() {
                if !next_lanes.iter().any(|l| l.as_ref().map_or(false, |ls| &ls.hash == p)) {
                    let color = if node_lane_idx == 0 { 0 } else if i == 0 { color_idx } else { self.next_color() };
                    let state = LaneState { hash: p.clone(), color_idx: color };
                    let pos = if i == 0 && node_lane_idx == 0 { 0 } 
                              else { next_lanes.iter().position(|l| l.is_none()).unwrap_or(next_lanes.len()) };
                    if pos >= next_lanes.len() { next_lanes.push(Some(state)); } 
                    else { next_lanes[pos] = Some(state); }
                }
            }

            rows.push(GraphRow {
                commit: commit.clone(),
                lanes: self.active_lanes.clone(),
                node_lane: node_lane_idx,
                node_color_idx: color_idx,
            });

            self.active_lanes = next_lanes;
            while self.active_lanes.last().map_or(false, |l| l.is_none()) { self.active_lanes.pop(); }
        }
        rows
    }

    fn next_color(&mut self) -> usize {
        let c = self.next_color_idx;
        self.next_color_idx = (self.next_color_idx + 1) % 6;
        if self.next_color_idx == 0 { self.next_color_idx = 1; }
        c
    }

    fn get_or_assign_lane(&mut self, hash: &str) -> usize {
        if let Some(pos) = self.active_lanes.iter().position(|l| l.as_ref().map_or(false, |ls| &ls.hash == hash)) {
            pos
        } else {
            let pos = if self.active_lanes.is_empty() { 0 } else { self.active_lanes.iter().position(|l| l.is_none()).unwrap_or(self.active_lanes.len()) };
            let color = if pos == 0 { 0 } else { self.next_color() };
            let state = LaneState { hash: hash.to_string(), color_idx: color };
            if pos >= self.active_lanes.len() { self.active_lanes.push(Some(state)); } 
            else { self.active_lanes[pos] = Some(state); }
            pos
        }
    }
}

pub struct GraphRow {
    pub commit: CommitInfo,
    pub lanes: Vec<Option<LaneState>>,
    pub node_lane: usize,
    pub node_color_idx: usize,
}

impl GraphRow {
    pub fn render(&self, is_selected: bool) -> Line {
        let mut spans = Vec::new();

        // 1. Graph prefix
        for (i, lane) in self.lanes.iter().enumerate() {
            if i == self.node_lane {
                let symbol = if self.node_lane == 0 { "○ " } else { "● " };
                spans.push(Span::styled(symbol, Style::default().fg(TuiGraphRenderer::get_lane_color(self.node_color_idx)).add_modifier(Modifier::BOLD)));
            } else if let Some(ls) = lane {
                spans.push(Span::styled("│ ", Style::default().fg(TuiGraphRenderer::get_lane_color(ls.color_idx))));
            } else {
                spans.push(Span::raw("  "));
            }
        }

        // 2. Padding to align
        let max_lanes = 4; // Compact
        if self.lanes.len() < max_lanes {
            for _ in 0..(max_lanes - self.lanes.len()) { spans.push(Span::raw("  ")); }
        }

        // 3. Subject
        let subject_style = if is_selected {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        spans.push(Span::styled(format!(" {:<40}", self.commit.subject), subject_style));

        // 4. Badges
        for label in &self.commit.labels {
            match label {
                LabelInfo::Head(n) => {
                    spans.push(Span::styled(format!(" HEAD → {} ", n), Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)));
                },
                LabelInfo::LocalBranch(n) => {
                     spans.push(Span::styled(format!(" {} ", n), Style::default().bg(Color::Blue).fg(Color::White)));
                },
                LabelInfo::RemoteBranch(n) => {
                     spans.push(Span::styled(format!(" origin/{} ", n), Style::default().bg(Color::Black).fg(Color::LightRed)));
                },
                LabelInfo::Tag(n) => {
                     spans.push(Span::styled(format!(" {} ", n), Style::default().bg(Color::Yellow).fg(Color::Black)));
                },
            }
            spans.push(Span::raw(" "));
        }

        Line::from(spans)
    }
}
