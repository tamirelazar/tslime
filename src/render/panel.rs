use super::theme::PanelStyle;

/// A panel with solid background and focus indicator.
pub struct Panel {
    /// X position of the panel.
    pub x: usize,
    /// Y position of the panel.
    pub y: usize,
    /// Width of the panel.
    pub width: usize,
    /// Height of the panel.
    pub height: usize,
    /// Panel styling.
    pub style: PanelStyle,
    /// Whether the panel is focused.
    pub focused: bool,
    /// Optional title for the panel.
    pub title: Option<String>,
    /// Content lines of the panel.
    pub content: Vec<String>,
}

impl Panel {
    /// Create a new panel.
    pub fn new(
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        style: PanelStyle,
        title: Option<String>,
    ) -> Self {
        Self {
            x,
            y,
            width,
            height,
            style,
            focused: false,
            title,
            content: Vec::new(),
        }
    }

    /// Set the focus state of the panel.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Set the content of the panel.
    pub fn set_content(&mut self, content: Vec<String>) {
        self.content = content;
    }

    /// Get the indicator color based on focus state.
    pub fn indicator_color(&self) -> &super::palette::RgbColor {
        if self.focused {
            &self.style.focus_color
        } else {
            &self.style.unfocus_color
        }
    }

    /// Render the panel content.
    pub fn render(&self) -> Vec<String> {
        let ind_w = self.style.indicator_width;
        let inner_width = self.width.saturating_sub(ind_w);

        let mut lines = Vec::with_capacity(self.height);

        if let Some(ref title) = self.title {
            let title_with_spaces = format!(" {} ", title);
            let title_len = title_with_spaces.chars().count();
            let remaining = inner_width.saturating_sub(2).saturating_sub(title_len);
            let left_pad = remaining / 2;
            let right_pad = remaining.saturating_sub(left_pad);

            let top_line = format!(
                "{}{}{}{}",
                " ".repeat(ind_w),
                " ".repeat(left_pad),
                title_with_spaces,
                " ".repeat(right_pad)
            );
            lines.push(top_line);
        } else {
            let top_line = " ".repeat(self.width);
            lines.push(top_line);
        }

        for (i, content_line) in self.content.iter().enumerate() {
            let content_len = content_line.chars().count();
            let right_pad = inner_width.saturating_sub(content_len);

            let line = format!(
                "{}{}{}",
                " ".repeat(ind_w),
                content_line,
                " ".repeat(right_pad)
            );
            lines.push(line);
        }

        let remaining_lines = self.height.saturating_sub(lines.len());
        for _ in 0..remaining_lines {
            lines.push(" ".repeat(self.width));
        }

        lines
    }
}
