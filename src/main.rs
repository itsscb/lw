use lw::App;

fn main() {
    let mut app = App::default();
    app.add("hello_world".into());
    app.save().unwrap();
    println!("{app:?}");
}
