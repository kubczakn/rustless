use std::{
  fs,
  io,
  error::Error, 
  thread,
  time
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
  terminal_state::{TerminalState, parse_input, ui},
  help
};

pub struct App {
  pub input: String, // either a file path or 'help'
}

impl App {
  pub fn build(
    mut args: impl Iterator<Item = String>,
  ) -> Result<App, &'static str> {

    args.next();

    let file_path = match args.next() {
      Some(arg) => arg,
      None => return Err(help::USAGE)
    };
  
    Ok(App{
      input: file_path,
    })
  }
}

pub fn run(config: App) -> Result<(), Box<dyn Error>> {
  // setup terminal
  enable_raw_mode()?;
  let mut stdout = io::stdout();
  execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  run_terminal(&mut terminal, config.input)?;

  // restore terminal 
  disable_raw_mode()?;
  execute!(
    terminal.backend_mut(),
    LeaveAlternateScreen,
    DisableMouseCapture
  )?;

  Ok(())
}

fn create_initial_display(input: String) -> String {
  let display_help_screen = input == "help";
  if display_help_screen {
    String::from(help::HELP)
  }
  else {
    fs::read_to_string(input).expect("Could not open file.")
  }
}

fn run_terminal<B: Backend>(terminal: &mut Terminal<B>, input : String) -> io::Result<()> {
  let ten_millis = time::Duration::from_millis(10);
  let mut terminal_state = TerminalState::new(create_initial_display(input));

  while terminal_state.running {
    terminal.draw(|f| ui(f, &terminal_state))?;

    if let Event::Key(key) = event::read()? {
      terminal_state = parse_input(key.code, terminal_state)    
    }

    if terminal_state.normal_mode() {
      terminal.hide_cursor().unwrap();
    }

    thread::sleep(ten_millis);
  }
  Ok(())
}
