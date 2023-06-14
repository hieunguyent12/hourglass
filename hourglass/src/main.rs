#[macro_use]
extern crate lazy_static;
extern crate dotenv;

use dotenv::dotenv;
use std::io;

mod app;
mod util;

use app::Hourglass;

fn main() -> io::Result<()> {
    dotenv().ok();

    let mut hourglass = Hourglass::new();
    hourglass.load_tasks()?;

    let mut terminal = Hourglass::start_tui()?;

    let r = hourglass.run(&mut terminal);
    Hourglass::pause_tui()?;
    r
}
