use ratatui::style::{palette::tailwind::SLATE, Color};

use ratatui::widgets::TableState;

const TEXT_FG_COLOR: Color = SLATE.c200;
const BG_COLOR: Color = SLATE.c900;

pub enum UIState {
    BudgrShow { state: TableState },
    LogShow(usize),
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
