use std::env;

use color_eyre::Result;
use lw::App;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut app = App::default();

    let args = env::args();

    if args.len() > 1 {
        args.into_iter().skip(1).for_each(|a| app.add(a.into()));
        app.save()?;
        return Ok(());
    }

    let terminal = ratatui::init();

    let result = app.run(terminal);
    ratatui::restore();
    result
}
