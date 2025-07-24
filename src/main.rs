use std::error::Error;

use lw::{App, log::Item};

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::default();
    let item = Item::from("hello_world");
    let id = item.id();
    app.add(item)?;
    println!("{app:?}");
    app.remove(id)?;
    app.save()?;
    println!("{app:?}");
    Ok(())
}
