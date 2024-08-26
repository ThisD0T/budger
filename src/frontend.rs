use crate::log::{read_budgr_from_directory, Budgr, Log, Purchase};
use color_eyre::Result;

use std::io::{stdout, Stdout};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{palette::tailwind::SLATE, Color},
    text::Line,
    widgets::{Block, ListItem, ListState, Padding, Paragraph, Widget},
    Frame, Terminal,
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
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl UI {
    pub fn new(terminal: Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<Self> {
        Ok(UI {
            state: UIState::Home,
            user_input: String::new(),
            input_mode: InputMode::Viewing,
            budgr: read_budgr_from_directory().unwrap(),
            terminal,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            let transition = match self.state {
                UIState::Home => self.render_logs(),
                UIState::LogView(i) => self.render_log(i),
            };

            match transition {
                Some(_) => (),
                None => (),
            };
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

impl UI {
    fn render_logs(&mut self) -> Option<UITransition> {
        self.terminal.draw(|f| render_logs_draw(f));

        None
    }

    fn render_log(&mut self, index: usize) -> Option<UITransition> {
        None
    }
}

fn render_logs_draw(frame: &mut Frame) {
    frame.render_widget(Paragraph::new("test paragraph"), frame.area());
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
