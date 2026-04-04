use crossterm::style::{Color, ContentStyle, Attribute, Stylize};

pub struct TreeStyle;

impl TreeStyle {
    pub fn head_node() -> ContentStyle {
        ContentStyle::new().with(Color::Cyan).attribute(Attribute::Bold)
    }

    pub fn commit_node() -> ContentStyle {
        ContentStyle::new().with(Color::Magenta)
    }

    pub fn merge_node() -> ContentStyle {
        ContentStyle::new().with(Color::DarkMagenta).attribute(Attribute::Bold)
    }

    pub fn connector() -> ContentStyle {
        // Very subtle dark gray
        ContentStyle::new().with(Color::AnsiValue(239))
    }

    pub fn hash() -> ContentStyle {
        ContentStyle::new().with(Color::AnsiValue(242))
    }

    pub fn subject() -> ContentStyle {
        ContentStyle::new().with(Color::White).attribute(Attribute::Bold)
    }

    pub fn metadata() -> ContentStyle {
        ContentStyle::new().with(Color::AnsiValue(238))
    }

    pub fn separator() -> ContentStyle {
        ContentStyle::new().with(Color::AnsiValue(236))
    }

    pub fn head_badge() -> ContentStyle {
        ContentStyle::new().with(Color::Cyan).attribute(Attribute::Bold)
    }

    pub fn local_branch_badge() -> ContentStyle {
        ContentStyle::new().with(Color::Green)
    }

    pub fn remote_branch_badge() -> ContentStyle {
        ContentStyle::new().with(Color::AnsiValue(167)) // Soft redish
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
