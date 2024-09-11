use ratatui::style::{palette::tailwind::SLATE, Color};

use ratatui::widgets::{ListState, TableState};

const TEXT_FG_COLOR: Color = SLATE.c200;
const BG_COLOR: Color = SLATE.c900;

pub enum UIState {
    BudgrShow { state: TableState },
    LogShow { index: usize, state: ListState },
    PurchaseInput(),
    Quit,
}

pub enum UITransition {
    OpenLog(usize),
    NewLog,
    ExitApp,
    ExitLayer,
    NewPurchase,
}

#[derive(Debug)]
pub enum UserInput {
    Next,
    Prev,
    Char(char),
    Submit,
    Esc,
    None,
}

pub enum InputMode {
    Editing,
    Viewing,
}
