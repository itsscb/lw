use color_eyre::Result;
use lw::App;

fn main() -> Result<()> {
    color_eyre::install()?;

    let terminal = ratatui::init();
    let mut app = App::default();

    let result = app.run(terminal);
    ratatui::restore();
    result
}
