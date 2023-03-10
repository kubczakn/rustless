use std::env;
use std::process;

use rustless::app::{
  App,
  run
};

fn main() {
  let config = App::build(env::args()).unwrap_or_else(|err| {
    eprintln!("{}", err);
    process::exit(1);
  });

  if let Err(err) = run(config) {
    eprintln!("Application error: {err}");
    process::exit(1);
  }
}
