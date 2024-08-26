use crate::frontend::UI;
mod frontend;
mod log;

use std::io::stdout;

use color_eyre::Result;

use ratatui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    crossterm::{execute, ExecutableCommand},
    layout::{Constraint, Layout, Rect},
    style::{palette::tailwind::SLATE, Color},
    text::Line,
    widgets::{Block, ListItem, ListState, Padding, Paragraph, Widget},
    Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn main() -> Result<()> {
    enable_raw_mode().unwrap();
    stdout().execute(EnterAlternateScreen)?;
    let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout())).unwrap();
    let mut ui = UI::new(terminal).unwrap();
    ui.run();

    Ok(())
}
