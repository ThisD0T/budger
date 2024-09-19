use crate::log::{Budgr, Log};
use crate::ui_data::{InputData, UIState, UITransition, UserInput};

use std::{io::Stdout, mem::swap};

use crossterm::{
    event::{self, Event, KeyCode},
    //terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Constraint,
    style::{palette::tailwind::SLATE, Color, Modifier, Style},
    text::Text,
    widgets::{Cell, ListItem, ListState, Row, Table, TableState},
    Terminal,
};

// style

const HIGHLIGHT_STYLE: Style = Style::new().add_modifier(Modifier::REVERSED).fg(SLATE.c500);
const ITEM_STYLE: Style = Style::new().fg(SLATE.c400);

pub struct UI {
    selection_index: usize,
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
        budgr: &Budgr,
    ) -> Option<UITransition> {
        match self {
            UIState::BudgrShow { state } => budgr_show(terminal, state, input, budgr),
            UIState::LogShow { index, state } => log_show(terminal, index, state, input, budgr),
            UIState::PurchaseInput { input_data, selection_index } => purchase_input(terminal, input_data, input, selection_index),
            _ => None,
        }
    }
}

impl UI {
    pub fn new(budgr: Budgr, terminal: Terminal<CrosstermBackend<Stdout>>) -> Self {
        UI {
            selection_index: 0,
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
                KeyCode::Up => UserInput::NextInput,
                KeyCode::Down => UserInput::PrevInput,
                KeyCode::Esc => UserInput::Esc,
                KeyCode::Backspace => UserInput::Backspace,
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
                // exit the app
                (UIState::BudgrShow { state: _ }, UITransition::ExitLayer) => {
                    self.run = false;
                }
                // open a log
                (UIState::BudgrShow { state: _ }, UITransition::OpenLog(i)) => {
                    self.state = UIState::LogShow {
                        index: i,
                        state: TableState::new(),
                    };
                    self.transition_flush();
                }
                (UIState::LogShow { index: i, state: _ }, UITransition::NewPurchase) => {}
                // go back to seeing all logs from log show
                (UIState::LogShow { index: _, state: _ }, UITransition::ExitLayer) => {
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
}

fn budgr_show(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    state: &mut TableState,
    input: &UserInput,
    budgr: &Budgr,
) -> Option<UITransition> {
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
        let colour = alternate_colour(&i);
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
    .highlight_style(HIGHLIGHT_STYLE);

    // draw them all in this closure
    let _ = terminal.draw(|frame| {
        frame.render_stateful_widget(table, frame.area(), state);
    });
    None
}

fn log_show(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    index: &mut usize,
    state: &mut TableState,
    input: &UserInput,
    budgr: &Budgr,
) -> Option<UITransition> {
    // input handle
    match input {
        //UserInput::Submit => return Some(UITransition::OpenLog(state.selected().unwrap())),
        UserInput::Esc => return Some(UITransition::ExitLayer),
        UserInput::Next => state.select_next(),
        UserInput::Prev => state.select_previous(),
        _ => {}
    }

    // make widgets
    let header = ["name", "purchase type", "cost"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(Style::new().fg(SLATE.c100).bg(SLATE.c950))
        .height(2);

    let rows = budgr.logs[*index]
        .purchases
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let item: [&String; 3] = [&p.name, &p.tag.to_string(), &p.cost.to_string()];

            item.into_iter()
                .map(|content| Cell::from(Text::from(format!("{content}"))))
                .collect::<Row>()
                .style(ITEM_STYLE.bg(alternate_colour(&i)))
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
    .highlight_style(HIGHLIGHT_STYLE);

    // render widgets
    terminal.draw(|frame| frame.render_stateful_widget(table, frame.area(), state));

    None
}

fn purchase_input( terminal: &mut Terminal<CrosstermBackend<Stdout>>,  dat: &mut InputData, input: &UserInput, selection_index: &mut u16) -> Option<UITransition> {
    // input handle
    match input {
        UserInput::Next => dat.move_cursor_right(),
        UserInput::Prev => dat.move_cursor_left(),
        UserInput::Esc => return Some(UITransition::ExitLayer),
        _ => (),
    }

    // make widgets
    // render
    None
}

fn alternate_colour(i: &usize) -> Color {
    match i % 2 {
        0 => SLATE.c800,
        _ => SLATE.c600,
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
