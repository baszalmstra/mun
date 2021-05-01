#[derive(Debug, Clone, Copy)]
pub enum DisplayColor {
    Disable,
    Auto,
    Enable,
}

impl DisplayColor {
    pub fn should_enable(self) -> bool {
        match self {
            DisplayColor::Disable => false,
            DisplayColor::Auto => console::colors_enabled(),
            DisplayColor::Enable => true,
        }
    }
}
