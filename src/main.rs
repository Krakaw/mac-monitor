extern crate clap;
extern crate ipnetwork;
extern crate pnet;

#[macro_use]
extern crate prettytable;

mod utils;

fn main() {
    let mut app = utils::cli::app::App::new();
    app.process();
}