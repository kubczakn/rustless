use std::{
  fs,
  io,
  io::{BufRead},
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

use crate::terminal_state::{TerminalState, parse_input, ui};

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

  Ok(())
}

fn run_terminal<B: Backend>(terminal: &mut Terminal<B>, file_path : String) -> io::Result<()> {
  // TODO: 
  //  - Jump to start and end of file  
  //  - Remember entered search patterns
  //  - Handle empty search pattern
  //  - Implement jump to next search pattern
  //      * Get line numbers for each match while styling
  //      * Contain text and match state within a type
  //      * Have an interface for types with text and match state

  let file = io::BufReader::new(fs::File::open(&file_path).expect("Could not open file."));
  let num_lines = file.lines().count() as u16;
  let content = fs::read_to_string(&file_path).expect("Could not open file.");

  let mut terminal_state = TerminalState::new(num_lines, &content);

  while terminal_state.running {
    terminal.draw(|f| ui(f, &terminal_state))?;

    if let Event::Key(key) = event::read()? {
      terminal_state = parse_input(key.code, terminal_state)    
    }

    if terminal_state.normal_mode {
      terminal.hide_cursor().unwrap();
    }
  }
  Ok(())
}
