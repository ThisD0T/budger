use crate::log::{read_budgr_from_directory, Budgr, Log, Purchase};
use crate::ui_data::{BudgrShowData, InputMode, LogShowData, UIState, UITransition, UserInput};
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
    style::{palette::tailwind::SLATE, Color, Style},
    text::{Line, Text},
    widgets::{
        Block, Cell, ListItem, ListState, Padding, Paragraph, Row, Table, TableState, Widget,
    },
    Frame, Terminal,
};

struct UI {
    selection_index: usize,
    character_pos: usize, // position of cursor for input
    input: String,
    user_input: UserInput,
    state: UIState,
    budgr: Budgr,
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl UI {
    pub fn new(budgr: Budgr, terminal: Terminal<CrosstermBackend<Stdout>>) -> Self {
        UI {
            selection_index: 0,
            character_pos: 0,
            input: String::new(),
            user_input: UserInput::None,
            state: UIState::BudgrShow,
            budgr,
            terminal,
        }
    }

    pub fn run(&mut self) {
        loop {
            self.process_input();
            self.transition();
        }
    }

    fn process_input(&mut self) {
        // process input

        if let Event::Key(key) = event::read().unwrap() {
            self.user_input = match key.code {
                KeyCode::Char(char) => UserInput::Char(char),
                KeyCode::Enter => UserInput::Submit,
                KeyCode::Left => UserInput::Prev,
                KeyCode::Right => UserInput::Next,
                KeyCode::Esc => UserInput::Esc,
                _ => UserInput::None,
            }
        }
    }
    fn transition(&mut self) {
        // draw then transition if needed

        if let Some(transition) = match self.state {
            UIState::BudgrShow => self.budgr_show(),
            UIState::LogShow(i) => self.log_show(i),
            UIState::PurchaseInput() => self.purchase_input(),
            _ => None,
        } {
            match (&self.state, transition) {
                (UIState::BudgrShow, UITransition::ExitApp) => {
                    self.state = UIState::Quit;
                    self.transition_flush();
                }
                (UIState::BudgrShow, UITransition::OpenLog(i)) => {
                    self.state = UIState::LogShow(i);
                    self.transition_flush();
                }
                (UIState::LogShow(_), UITransition::ExitLayer) => {
                    self.state = UIState::BudgrShow;
                    self.transition_flush();
                }
                (_, _) => (),
            }
        }
    }

    fn transition_flush(&mut self) {
        self.input.clear();
        self.selection_index = 0;
    }

    // UIState methods
    fn exit_app(&mut self) {}
    fn budgr_show(&mut self, dat: BudgrShowData) -> Option<UITransition> {
        // handle inputs
        if let Some(t) = self.standard_input_handle() {
            return Some(t);
        }

        match self.user_input {
            UserInput::Submit => return Some(UITransition::OpenLog(dat.state.selected().unwrap())),
            _ => {}
        }

        // make a bunch of widgets to draw

        // table
        let header = ["log name", "num purchases", "total expense"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(Style::new().fg(SLATE.c100).bg(SLATE.c950))
            .height(4);

        let rows = self.budgr.logs.iter().enumerate().map(|(i, log)| {
            let colour = match i % 2 {
                0 => SLATE.c900,
                _ => SLATE.c800,
            };
            let item: [&String; 3] = [
                &log.name,
                &log.purchases.len().to_string(),
                &log.get_total().to_string(),
            ];
            item.into_iter()
                .map(|content| Cell::from(Text::from(format!("{content}"))))
                .collect::<Row>()
                .style(Style::new().fg(SLATE.c100).bg(colour))
                .height(4);
        });

        let table = Table::new(rows).header(header);

        // draw them all in this closure
        self.terminal.draw(|frame| {});
        None
    }

    fn log_show(&mut self, index: usize) -> Option<UITransition> {
        None
    }

    fn purchase_input(&mut self) -> Option<UITransition> {
        None
    }

    // if standard_input_handle returns Some(foo) it means the user wishes to exit the layer
    fn standard_input_handle(&mut self) -> Option<UITransition> {
        match self.user_input {
            UserInput::Prev => self.selection_index -= 1,
            UserInput::Next => self.selection_index += 1,
            UserInput::Esc => return Some(UITransition::ExitLayer),
            _ => {}
        }
        return None;
    }

    // - - - string input stuff - - -
    // mostly stolen from ratatui example

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_pos.saturating_sub(1);
        self.character_pos = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_pos.saturating_add(1);
        self.character_pos = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_pos)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        // TODO: reverse this if statement
        let is_not_cursor_leftmost = self.character_pos != 0;

        // we are creating 2 iterators over that skip over the character that the curosr is over then combining them to delete the char
        if is_not_cursor_leftmost {
            let current_index = self.character_pos;
            let from_left_to_current_index = current_index - 1;
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn reset_cursor(&mut self) {
        self.character_pos = 0;
    }
}

impl From<&Log> for ListItem<'_> {
    fn from(log: &Log) -> Self {
        ListItem::new(format!(
            "{}: # Purchases: {} Total Value: {}",
            log.name,
            log.purchases.len(),
            log.get_total()
        ))
    }
}
