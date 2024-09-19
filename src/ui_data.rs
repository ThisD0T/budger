use crate::frontend::UI;
use ratatui::style::{palette::tailwind::SLATE, Color};
use ratatui::widgets::{ListState, TableState};

const TEXT_FG_COLOR: Color = SLATE.c200;
const BG_COLOR: Color = SLATE.c900;

pub enum UIState {
    BudgrShow { state: TableState },
    LogShow { index: usize, state: TableState },
    PurchaseInput { input_data: Vec<InputData>, selection_index: usize, log_index: usize },
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
    NextSelect,
    PrevSelect,
    Char(char),
    Submit,
    Esc,
    Backspace,
    None,
}

pub enum InputMode {
    Editing,
    Viewing,
}

// data needed to create an input box
#[derive(Clone)]
pub struct InputData {
    pub input: String,
    pub character_pos: usize,
}

impl InputData {
    // - - - string input stuff - - -
    // mostly stolen from ratatui example

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_pos.saturating_sub(1);
        self.character_pos = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_pos.saturating_add(1);
        self.character_pos = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_pos)
            .unwrap_or(self.input.len())
    }

    pub fn delete_char(&mut self) {
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
