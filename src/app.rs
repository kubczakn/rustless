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

use crate::terminal_state::{TerminalState, parse_input, ui};

pub struct App {
  pub file_path: String,
}

impl App {
  pub fn build(
    mut args: impl Iterator<Item = String>,
  ) -> Result<App, &'static str> {

    args.next();

    let file_path = match args.next() {
      Some(arg) => arg,
      None => return Err("Didn't get a filepath.")
    };
  
    Ok(App{
      file_path,
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
  //  - Ability to change files
  //  - Add 'help' option to display navigation and command options

  let content = fs::read_to_string(&file_path).expect("Could not open file.");
  let mut terminal_state = TerminalState::new(&content);
  let ten_millis = time::Duration::from_millis(10);

  while terminal_state.running {
    terminal.draw(|f| ui(f, &terminal_state))?;

    if let Event::Key(key) = event::read()? {
      terminal_state = parse_input(key.code, terminal_state)    
    }

    if terminal_state.normal_mode {
      terminal.hide_cursor().unwrap();
    }

    thread::sleep(ten_millis);
  }
  Ok(())
}
