mod log;
use log::*;

fn main() {
    let mut budgr: Budgr = read_budgr_from_directory().unwrap();

    budgr.serialize().unwrap();
}
