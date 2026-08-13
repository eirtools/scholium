use core::fmt::Display;

use nu_ansi_term::{Color, Style};

pub(super) struct Styled<'a> {
    is_ansi: bool,
    input: &'a str,
    style: Style,
}

impl<'a> Styled<'a> {
    pub fn new(is_ansi: bool, input: &'a str, style: Style) -> Self {
        Self {
            is_ansi,
            input,
            style,
        }
    }

    pub fn prefix(is_ansi: bool, input: &'a str) -> Self {
        Self::new(is_ansi, input, Color::LightBlue.bold())
    }
}

impl<'a> Display for Styled<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_ansi {
            write!(f, "{}", self.style.paint(self.input))
        } else {
            write!(f, "{}", self.input)
        }
    }
}
