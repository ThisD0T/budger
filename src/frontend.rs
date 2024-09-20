use crate::log::{Budgr, Log};
use crate::ui_data::{InputData, UIState, UITransition, UserInput};

use std::{io::Stdout, mem::swap};

use crossterm::{
    event::{self, Event, KeyCode},
    //terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::style::Stylize;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Direction, Constraint::Ratio},
    style::{palette::tailwind::SLATE, Color, Modifier, Style},
    text::Text,
    widgets::{Cell, ListItem, Row, Table, TableState, Paragraph},
    Terminal,
};

// style

const HIGHLIGHT_STYLE: Style = Style::new().add_modifier(Modifier::REVERSED).fg(SLATE.c900).bg(SLATE.c100);
const ITEM_STYLE: Style = Style::new().fg(SLATE.c100).bg(SLATE.c900);

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
        budgr: &mut Budgr,
    ) -> Option<UITransition> {
        match self {
            UIState::BudgrShow { state } => budgr_show(terminal, state, input, budgr),
            UIState::LogShow { index, state } => log_show(terminal, index, state, input, budgr),
            UIState::PurchaseInput { input_data, selection_index, log_index } => purchase_input(terminal, input_data, input, selection_index, *log_index, budgr),
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
            self.transition();
            self.process_input();
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
                KeyCode::Down=> UserInput::NextSelect,
                KeyCode::Up => UserInput::PrevSelect,
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
                // create a new purchase
                (UIState::LogShow { index: i, state: _ }, UITransition::NewPurchase) => {
                    self.state = UIState::PurchaseInput {
                        input_data: vec![InputData{input: String::new(), character_pos: 0}; 3],
                        selection_index: 0,
                        log_index: *i,
                    }
                }
                // go back to seeing all logs from log show
                (UIState::LogShow { index: _, state: _ }, UITransition::ExitLayer) => {
                    self.state = UIState::BudgrShow {
                        state: TableState::new(),
                    };
                    self.transition_flush();
                }
                (UIState::PurchaseInput { input_data: _, selection_index: _, log_index }, UITransition::ExitLayer) => {
                    self.state = UIState::LogShow{index: *log_index, state: TableState::new()};
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
        UserInput::NextSelect => state.select_next(),
        UserInput::PrevSelect => state.select_previous(),
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
    budgr: &mut Budgr,
) -> Option<UITransition> {
    // input handle
    match input {
        //UserInput::Submit => return Some(UITransition::OpenLog(state.selected().unwrap())),
        UserInput::Esc => return Some(UITransition::ExitLayer),
        UserInput::NextSelect => state.select_next(),
        UserInput::PrevSelect => state.select_previous(),
        UserInput::Char('a') => return Some(UITransition::NewPurchase),
        UserInput::Char('d') => {
            budgr.remove_purchase(*index, state.selected()?);
            return None
        }
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
            let item: [&String; 2] = [&p.name, &p.cost.to_string()];

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

fn purchase_input( terminal: &mut Terminal<CrosstermBackend<Stdout>>,  dat: &mut Vec<InputData>, input: &UserInput, selection_index: &mut usize, log_index: usize, budgr: &mut Budgr) -> Option<UITransition> {
    // input handle
    match input {
        UserInput::Next => dat[*selection_index].move_cursor_right(),
        UserInput::Prev => dat[*selection_index].move_cursor_left(),
        UserInput::NextSelect => {
            if *selection_index < dat.len() {
                *selection_index += 1
            }
        }
        UserInput::PrevSelect => {
            if *selection_index > 0 {
                *selection_index -= 1
            }
        }
        UserInput::Char(c) => dat[*selection_index].enter_char(*c),
        UserInput::Esc => return Some(UITransition::ExitLayer),
        UserInput::Submit => {
            match selection_index {
                // attempt to create a new purchase
                2 => {
                    let cost: i64;
                    if let Ok(int) = dat[1].input.parse::<i64>() {
                        cost = int;
                    } else {
                        // TODO: error handling (don't just spit the user out of the menu if they
                        // inputted something incorrectly)

                        return Some(UITransition::ExitLayer);
                    };

                    budgr.add_purchase(log_index, dat[0].input.clone(), cost);
                }
                _ => (),
            };
        }
        _ => (),
    }


    // make widgets
    
    let mut name_input = Paragraph::new(dat[0].input.as_str());
    let mut cost_input = Paragraph::new(dat[1].input.as_str());
    let mut submit_button = Paragraph::new("Submit");
    
    match selection_index {
        0 => {
            name_input = name_input.style(HIGHLIGHT_STYLE).add_modifier(Modifier::BOLD);
        }
        1 => {
            cost_input = cost_input.style(HIGHLIGHT_STYLE).add_modifier(Modifier::BOLD);
        }
        2 => {
            submit_button = submit_button.style(HIGHLIGHT_STYLE).add_modifier(Modifier::BOLD);
        }
        _ => (),
    };

    // render
    
    let _ = terminal.draw(| f | {
        let layout = Layout::vertical([Ratio(1, 3); 3]);
        let [name_area, cost_area, submit_area] = layout.areas(f.area());

        f.render_widget(name_input, name_area);
        f.render_widget(cost_input, cost_area);
        f.render_widget(submit_button, submit_area);
    });

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
