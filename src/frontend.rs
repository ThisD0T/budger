use crate::log::{read_budgr_from_directory, Budgr, Log, Purchase};
use color_eyre::Result;

use ratatui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
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

const TEXT_FG_COLOR: Color = SLATE.c200;

fn test_funccy() {
    println!("poggers");
}

enum InputMode {
    Viewing,
    Editing,
}

// essentially a screen that will take up the whole of the terminal interface
pub enum UIState {
    Home,           // view all logs
    LogView(usize), // view specific log
}

enum UITransition {
    ExitApp,
    ExitLayer,
    OpenLog(usize),
    NewLog,
    NewPurchase,
}

pub struct UI {
    state: UIState,
    user_input: String,
    input_mode: InputMode,
    budgr: Budgr,
}

impl UI {
    pub fn new() -> Result<Self> {
        Ok(UI {
            state: UIState::Home,
            user_input: String::new(),
            input_mode: InputMode::Viewing,
            budgr: read_budgr_from_directory().unwrap(),
        })
    }

    pub fn run(&self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
        loop {
            // UIState is a function that returns Some<UITransition>, if Some, transition UI
            terminal.draw(|frame| frame.render_widget(self, frame.area()))?;
        }
    }

    fn transition(&mut self, transition: UITransition) {
        match (&self.state, transition) {
            (UIState::Home, UITransition::ExitApp) => self.exit(),
            (UIState::Home, UITransition::OpenLog(index)) => self.state = UIState::LogView(index),
            (UIState::LogView(_), UITransition::ExitLayer) => self.state = UIState::Home,
            (_, _) => (),
        }
    }

    fn exit(&mut self) {}
}

impl Widget for &UI {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.state {
            UIState::Home => self.render_logs(area, buf),
            UIState::LogView(index) => self.render_log(area, buf, index),
        }
    }
}

impl UI {
    fn render_logs(&mut self, area: Rect, buf: &mut Buffer) {}

    fn render_log(&mut self, area: Rect, buf: &mut Buffer, index: usize) {}
}

// - - - - - trait bullshit - - - - -
#[derive(Debug)]
struct LogsList {
    items: Vec<Log>,
    state: ListState,
}

// formatting a log to be a list item for the widget
impl From<&Log> for ListItem<'_> {
    fn from(log: &Log) -> Self {
        ListItem::new(Line::styled(
            format!("{}, purchase #: {}", log.name, log.purchases.len()),
            TEXT_FG_COLOR,
        ))
    }
}
