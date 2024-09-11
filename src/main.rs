use crate::frontend::UI;
mod frontend;
mod log;
mod ui_data;

use log::{Budgr, PurchaseType};

use color_eyre::Result;

use log::read_budgr_from_directory;

fn main() -> Result<()> {
    //stdout().execute(EnterAlternateScreen)?;
    //let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout())).unwrap();

    let terminal = ratatui::init();
    let budgr = read_budgr_from_directory().unwrap();
    let mut ui = UI::new(budgr, terminal);
    ui.run();

    Ok(())
}

fn make_test_budgr() -> Budgr {
    let mut budgr = Budgr::new();
    for i in 0..4 {
        let _ = budgr.new_log(format!("test_log{}", i));
        for j in 0..4 {
            let _ = budgr.add_purchase(
                i,
                format!("purchase:{}", (i * j) as i64),
                PurchaseType::Groceries,
                (i * j) as i64,
            );
        }
    }
    budgr
}
