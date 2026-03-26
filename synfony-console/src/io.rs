use comfy_table::{Table, presets::UTF8_FULL};
use console::Term;
use dialoguer::{Confirm, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};

use crate::style::ConsoleStyle;

/// Symfony-style console I/O helper.
///
/// Provides a familiar API for console output that mirrors Symfony's `SymfonyStyle`:
/// - `title()`, `section()` — headers
/// - `success()`, `error()`, `warning()`, `info()` — styled messages
/// - `ask()`, `confirm()`, `choice()` — interactive prompts
/// - `table()` — formatted table output
/// - `progress_bar()` — progress indicators
pub struct ConsoleIO {
    _term: Term,
}

impl ConsoleIO {
    pub fn new() -> Self {
        ConsoleIO {
            _term: Term::stdout(),
        }
    }

    pub fn title(&self, text: &str) {
        let style = ConsoleStyle::title();
        println!();
        println!(" {}", style.apply_to(text));
        println!(" {}", style.apply_to("=".repeat(text.len())));
        println!();
    }

    pub fn section(&self, text: &str) {
        let style = ConsoleStyle::section();
        println!();
        println!(" {}", style.apply_to(text));
        println!(" {}", style.apply_to("-".repeat(text.len())));
        println!();
    }

    pub fn success(&self, text: &str) {
        let style = ConsoleStyle::success();
        println!();
        println!(" {} {}", style.apply_to("[OK]"), text);
        println!();
    }

    pub fn error(&self, text: &str) {
        let style = ConsoleStyle::error();
        eprintln!();
        eprintln!(" {} {}", style.apply_to("[ERROR]"), text);
        eprintln!();
    }

    pub fn warning(&self, text: &str) {
        let style = ConsoleStyle::warning();
        println!();
        println!(" {} {}", style.apply_to("[WARNING]"), text);
        println!();
    }

    pub fn info(&self, text: &str) {
        let style = ConsoleStyle::info();
        println!(" {} {}", style.apply_to("[INFO]"), text);
    }

    pub fn comment(&self, text: &str) {
        let style = ConsoleStyle::comment();
        println!(" {}", style.apply_to(text));
    }

    pub fn newline(&self) {
        println!();
    }

    pub fn table(&self, headers: Vec<&str>, rows: Vec<Vec<&str>>) {
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(headers);
        for row in rows {
            table.add_row(row);
        }
        println!("{table}");
    }

    pub fn progress_bar(&self, total: u64) -> ProgressBar {
        let pb = ProgressBar::new(total);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(" {spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("=>-"),
        );
        pb
    }

    pub fn spinner(&self, message: &str) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template(" {spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        pb
    }

    pub fn ask(&self, question: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(Input::new()
            .with_prompt(question)
            .interact_text()?)
    }

    pub fn ask_with_default(
        &self,
        question: &str,
        default: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        Ok(Input::new()
            .with_prompt(question)
            .default(default.to_string())
            .interact_text()?)
    }

    pub fn confirm(&self, question: &str, default: bool) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(Confirm::new()
            .with_prompt(question)
            .default(default)
            .interact()?)
    }

    pub fn choice(&self, question: &str, options: &[&str]) -> Result<usize, Box<dyn std::error::Error>> {
        Ok(Select::new()
            .with_prompt(question)
            .items(options)
            .interact()?)
    }

    pub fn definition_list(&self, items: &[(&str, &str)]) {
        let label_style = ConsoleStyle::label();
        for (key, value) in items {
            println!(" {} {}", label_style.apply_to(format!("{key}:")), value);
        }
    }
}

impl Default for ConsoleIO {
    fn default() -> Self {
        Self::new()
    }
}
