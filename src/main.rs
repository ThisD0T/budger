use crate::frontend::UI;
mod frontend;
mod log;

use ratatui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{palette::tailwind::SLATE, Color},
    text::Line,
    widgets::{Block, ListItem, ListState, Padding, Paragraph, Widget},
    Terminal,
};

fn main() {
    let mut term = Terminal::new();
}
