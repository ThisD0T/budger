use crate::log::{Budgr, Log};
use crate::ui_data::{UIState, UITransition, UserInput};

use std::{io::Stdout, mem::swap};

use crossterm::{
    event::{self, Event, KeyCode},
    //terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Constraint,
    style::{palette::tailwind::SLATE, Modifier, Style},
    text::Text,
    widgets::{Cell, ListItem, Row, Table, TableState},
    Terminal,
};

pub struct UI {
    selection_index: usize,
    character_pos: usize, // position of cursor for input
    input: String,
    user_input: UserInput,
    state: UIState,
    budgr: Budgr,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    run: bool,
}

impl UIState {
    fn render(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        input: &UserInput,
        budgr: &mut Budgr,
    ) -> Option<UITransition> {
        match self {
            UIState::BudgrShow { state } => UIState::budgr_show(terminal, state, input, budgr),
            _ => None,
        }
    }

    fn budgr_show(
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        state: &mut TableState,
        input: &UserInput,
        budgr: &mut Budgr,
    ) -> Option<UITransition> {
        let highlight_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(SLATE.c500);

        // handle inputs
        match input {
            UserInput::Submit => return Some(UITransition::OpenLog(state.selected().unwrap())),
            UserInput::Esc => return Some(UITransition::ExitLayer),
            UserInput::Next => state.select_next(),
            UserInput::Prev => state.select_previous(),
            _ => {}
        }

        // make a bunch of widgets to draw

        // table widget
        let header = ["log name", "num purchases", "total expense"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(Style::new().fg(SLATE.c100).bg(SLATE.c950))
            .height(2);

        let rows = budgr.logs.iter().enumerate().map(|(i, log)| {
            let colour = match i % 2 {
                0 => SLATE.c800,
                _ => SLATE.c600,
            };
            let item: [&String; 3] = [
                &log.name,
                &log.purchases.len().to_string(),
                &log.get_total().to_string(),
            ];
            item.into_iter()
                .map(|content| Cell::from(Text::from(format!("{content}"))))
                .collect::<Row>()
                .style(Style::new().fg(SLATE.c400).bg(colour))
                .height(4)
        });

        let table = Table::new(
            rows,
            [
                Constraint::Length(64),
                Constraint::Min(26),
                Constraint::Min(25),
            ],
        )
        .header(header)
        .highlight_style(highlight_style);

        // draw them all in this closure
        let _ = terminal.draw(|frame| {
            frame.render_stateful_widget(table, frame.area(), state);
        });
        None
    }
}

impl UI {
    pub fn new(budgr: Budgr, terminal: Terminal<CrosstermBackend<Stdout>>) -> Self {
        UI {
            selection_index: 0,
            character_pos: 0,
            input: String::new(),
            user_input: UserInput::None,
            state: UIState::BudgrShow {
                state: TableState::new(),
            },
            budgr,
            terminal,
            run: true,
        }
    }

    pub fn run(&mut self) {
        while self.run {
            self.process_input();
            self.transition();
        }
        self.budgr.serialize();
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

        if let Some(transition) =
            self.state
                .render(&mut self.terminal, &mut self.user_input, &mut self.budgr)
        {
            // transition if needed
            match (&self.state, transition) {
                (UIState::BudgrShow { state: _ }, UITransition::ExitLayer) => {
                    self.run = false;
                }
                (UIState::BudgrShow { state: _ }, UITransition::OpenLog(i)) => {
                    self.state = UIState::LogShow(i);
                    self.transition_flush();
                }
                (UIState::LogShow(_), UITransition::ExitLayer) => {
                    self.state = UIState::BudgrShow {
                        state: TableState::new(),
                    };
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
