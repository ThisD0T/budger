use ratatui::style::{palette::tailwind::SLATE, Color};

use ratatui::widgets::TableState;

const TEXT_FG_COLOR: Color = SLATE.c200;
const BG_COLOR: Color = SLATE.c900;

// state data
pub struct LogShowData {
    index: usize,
    pub state: TableState,
}

pub struct BudgrShowData {
    pub state: TableState,
}

pub enum UIState {
    BudgrShow,
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
