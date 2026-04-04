use crossterm::style::{Color, ContentStyle, Attribute, Stylize};

pub struct TreeStyle;

impl TreeStyle {
    /// The primary color for HEAD commit nodes
    pub fn head_node() -> ContentStyle {
        ContentStyle::new().with(Color::Cyan).attribute(Attribute::Bold)
    }

    /// Normal commit node color
    pub fn commit_node() -> ContentStyle {
        ContentStyle::new().with(Color::Magenta)
    }

    /// Merge commit node (distinguishable by styling)
    pub fn merge_node() -> ContentStyle {
        ContentStyle::new().with(Color::DarkMagenta).attribute(Attribute::Bold)
    }

    /// Subdued connector line color (IDE-style gray)
    pub fn connector() -> ContentStyle {
        ContentStyle::new().with(Color::DarkGrey)
    }

    /// Dimmed hash color
    pub fn hash() -> ContentStyle {
        ContentStyle::new().with(Color::DarkGrey)
    }

    /// Focal point: bold subject
    pub fn subject() -> ContentStyle {
        ContentStyle::new().attribute(Attribute::Bold)
    }

    /// Dimmed metadata (author, date)
    pub fn metadata() -> ContentStyle {
        ContentStyle::new().with(Color::DarkGrey)
    }

    /// Separator dot / bullet
    pub fn separator() -> ContentStyle {
        ContentStyle::new().with(Color::DarkGrey)
    }

    // --- [ Badges / Refs ] ---

    /// HEAD -> main emphasis
    pub fn head_badge() -> ContentStyle {
        ContentStyle::new().with(Color::Cyan).attribute(Attribute::Bold)
    }

    /// Local branch badge style
    pub fn local_branch_badge() -> ContentStyle {
        ContentStyle::new().with(Color::Green)
    }

    /// Remote branch style
    pub fn remote_branch_badge() -> ContentStyle {
        ContentStyle::new().with(Color::DarkRed)
    }

    /// Tag badge style
    pub fn tag_badge() -> ContentStyle {
        ContentStyle::new().with(Color::Yellow)
    }

    /// Pipe separator for multiple refs
    pub fn ref_divider() -> ContentStyle {
        ContentStyle::new().with(Color::DarkGrey)
    }
}

/// Helper to wrap text with a style
pub fn styled<D: std::fmt::Display>(content: D, style: ContentStyle) -> String {
    format!("{}", style.apply(content))
}
