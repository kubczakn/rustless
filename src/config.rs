use std::{
  io,
  error::Error, 
};
use tui::{
  backend::{CrosstermBackend, Backend},
  Terminal,
};
use crossterm::{
  event::{self, DisableMouseCapture, EnableMouseCapture, Event},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::{
  view_manager::ViewManager
};

pub struct Config {
  pub file_path: String,
}

impl Config {
  pub fn build(
    mut args: impl Iterator<Item = String>,
  ) -> Result<Config, &'static str> {

    args.next();

    let file_path = match args.next() {
      Some(arg) => arg,
      None => return Err("Didn't get a filepath.")
    };
  
    Ok(Config{
      file_path,
    })
  }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
  // setup terminal
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let _res = run_terminal(&mut terminal, config.file_path);

  // restore terminal 
  disable_raw_mode()?;
  execute!(
    terminal.backend_mut(),
    LeaveAlternateScreen,
    DisableMouseCapture
  )?;
  terminal.show_cursor().unwrap();

  Ok(())
}

fn run_terminal<B: Backend>(terminal: &mut Terminal<B>, file_path : String) -> io::Result<()> {
  let mut view_manager = ViewManager::new(file_path);

  while view_manager.running() {
    terminal.draw(|f| view_manager.ui(f))?;
    if let Event::Key(key) = event::read()? {
      view_manager.parse_input(key.code) 
    }
  }

  Ok(())
}
