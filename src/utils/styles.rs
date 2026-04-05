use crossterm::style::{Attribute, Color, ContentStyle, Stylize};

pub struct TreeStyle;

impl TreeStyle {
    pub fn head_node() -> ContentStyle {
        ContentStyle::new()
            .with(Color::Cyan)
            .attribute(Attribute::Bold)
    }

    pub fn commit_node(lane: usize) -> ContentStyle {
        ContentStyle::new().with(Self::lane_color(lane))
    }

    pub fn merge_node(lane: usize) -> ContentStyle {
        ContentStyle::new()
            .with(Self::lane_color(lane))
            .attribute(Attribute::Bold)
    }

    pub fn connector(lane: usize) -> ContentStyle {
        ContentStyle::new().with(Self::lane_color(lane))
    }

    pub fn lane_color(lane: usize) -> Color {
        match lane % 6 {
            0 => Color::Blue,
            1 => Color::Green,
            2 => Color::Yellow,
            3 => Color::Magenta,
            4 => Color::Cyan,
            _ => Color::Red,
        }
    }

    pub fn hash() -> ContentStyle {
        ContentStyle::new().with(Color::AnsiValue(242))
    }

    pub fn subject() -> ContentStyle {
        ContentStyle::new()
            .with(Color::White)
            .attribute(Attribute::Bold)
    }

    pub fn metadata() -> ContentStyle {
        // Very dim gray for metadata
        ContentStyle::new().with(Color::AnsiValue(238))
    }

    pub fn separator() -> ContentStyle {
        ContentStyle::new().with(Color::AnsiValue(236))
    }

    // --- Badges ---

    pub fn head_badge() -> ContentStyle {
        ContentStyle::new()
            .with(Color::Cyan)
            .attribute(Attribute::Bold)
    }

    pub fn local_branch_badge() -> ContentStyle {
        ContentStyle::new().with(Color::Green)
    }

    pub fn remote_branch_badge() -> ContentStyle {
        ContentStyle::new().with(Color::AnsiValue(167))
    }

    pub fn tag_badge() -> ContentStyle {
        ContentStyle::new().with(Color::Yellow)
    }

    pub fn ref_divider() -> ContentStyle {
        ContentStyle::new().with(Color::AnsiValue(236))
    }
}

pub fn styled<D: std::fmt::Display>(content: D, style: ContentStyle) -> String {
    format!("{}", style.apply(content))
}
