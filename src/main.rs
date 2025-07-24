use color_eyre::Result;
use lw::{App, log::Item};

fn main() -> Result<()> {
    color_eyre::install()?;

    let terminal = ratatui::init();
    let mut app = App::default();

    let result = app.run(terminal);
    ratatui::restore();
    result
}
