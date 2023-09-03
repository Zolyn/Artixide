use std::process::{Command, Output};

use color_eyre::{eyre::eyre, Help, Report, Result, SectionExt};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{
    style::{Color, Style},
    widgets::{Block, Borders},
};
use sealed::sealed;

use crate::lazy;

lazy! {
    pub static FUZZY_MATCHER: SkimMatcherV2 = SkimMatcherV2::default();
}

#[sealed]
pub trait CommandExt {
    fn run(&mut self) -> Result<()>;
    fn read(&mut self) -> Result<String>;
}

#[sealed]
impl CommandExt for Command {
    fn run(&mut self) -> Result<()> {
        let Output {
            status,
            stderr,
            stdout,
        } = self.output()?;

        if status.success() {
            return Ok(());
        }

        Err(wrap_command_error(&stdout, &stderr))
    }

    fn read(&mut self) -> Result<String> {
        let Output {
            status,
            stderr,
            stdout,
        } = self.output()?;

        if status.success() {
            return Ok(String::from_utf8_lossy(&stdout).into());
        }

        Err(wrap_command_error(&stdout, &stderr))
    }
}

pub fn wrap_command_error(stdout: &[u8], stderr: &[u8]) -> Report {
    let stdout = String::from_utf8_lossy(stdout);
    let stderr = String::from_utf8_lossy(stderr);

    eyre!("Command execution failure")
        .with_section(|| stdout.trim().to_string().header("Stdout"))
        .with_section(|| stderr.trim().to_string().header("Stderr"))
}

#[sealed]
pub trait StrExt {
    fn fuzzy_indices(&self, choice: &str) -> Option<Vec<usize>>;
    fn slice(&self, start: usize, end: usize) -> Option<&str>;
}

#[sealed]
impl StrExt for str {
    fn fuzzy_indices(&self, choice: &str) -> Option<Vec<usize>> {
        FUZZY_MATCHER
            .fuzzy_indices(choice, self)
            .map(|(_, indices)| indices)
    }

    fn slice(&self, start: usize, end: usize) -> Option<&str> {
        if start >= end {
            return None;
        }

        let mut indices = self
            .char_indices()
            .map(|(index, _)| index)
            .chain(std::iter::once_with(|| self.len()));

        let start_index = indices.nth(start)?;

        let end_index = indices.nth(end - start - 1)?;

        Some(&self[start_index..end_index])
    }
}

#[sealed]
pub trait BlockExt {
    fn with_borders() -> Block<'static> {
        Block::default()
            .borders(Borders::all())
            .style(Style::with_fg())
    }
}

#[sealed]
impl BlockExt for Block<'_> {}

#[sealed]
pub trait StyleExt {
    fn with_fg() -> Style {
        Style::default().fg(Color::Gray)
    }
}

#[sealed]
impl StyleExt for Style {}

pub trait Take: Default {
    fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

impl Take for String {}
