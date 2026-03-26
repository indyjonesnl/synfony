use console::Style;

/// Pre-defined styles matching Symfony Console's SymfonyStyle output.
pub struct ConsoleStyle;

impl ConsoleStyle {
    pub fn title() -> Style {
        Style::new().bold().cyan()
    }

    pub fn section() -> Style {
        Style::new().bold().yellow()
    }

    pub fn success() -> Style {
        Style::new().bold().green()
    }

    pub fn error() -> Style {
        Style::new().bold().red()
    }

    pub fn warning() -> Style {
        Style::new().bold().yellow()
    }

    pub fn info() -> Style {
        Style::new().bold().blue()
    }

    pub fn comment() -> Style {
        Style::new().dim()
    }

    pub fn label() -> Style {
        Style::new().bold()
    }
}
