use std::{
  fs,
  io,
  io::BufRead,
};
use tui::{
  backend::{Backend},
  widgets::{Paragraph},
  layout::{Layout, Constraint, Direction},
  text::{Text},
  Frame 
};
use crossterm::{
  event::{KeyCode},
};

const SCREEN_END_OFFSET: u16 = 10;

enum InputState {
  Normal,
  Command,
}

struct FileContent {
  num_lines: u16,
  content: String
}

impl FileContent {
  fn new(file_path: String) -> FileContent {
    let file = io::BufReader::new(fs::File::open(&file_path).expect("Could not open file."));
    FileContent {
      num_lines: file.lines().count() as u16,
      content: fs::read_to_string(&file_path).expect("Could not open file.")
    }
  }

  fn num_lines(&self) -> u16 {
    self.num_lines
  }

  fn content(&self) -> &str {
    &self.content
  }

}

struct CommandPrompt {
  command: String,
}

impl CommandPrompt {
  fn new() -> CommandPrompt {
    CommandPrompt {
      command: String::from(":"),
    }
  }

  fn change_prompt(&mut self, prompt_in: char) {
    self.command.clear();
    self.command.push(prompt_in)
  }

  fn command(&self) -> &str {
    &self.command
  }

  fn pop_back(& mut self) {
    if self.command.len() > 1 {
      self.command.pop();
    }
  }

  fn parse(&mut self, key: KeyCode) {
    match key {
      KeyCode::Char(char_in) => self.command.push(char_in),
      KeyCode::Backspace => self.pop_back(),
      _ => ()
    }
  }
}

pub struct ViewManager {
  content: FileContent,
  command_prompt: CommandPrompt, 
  scroll_offset: u16,
  state: InputState,
  running: bool
}

impl ViewManager {
  pub fn new(file_path: String) -> ViewManager {
    ViewManager {
      content: FileContent::new(file_path), 
      command_prompt: CommandPrompt::new(),
      scroll_offset: 0,
      state: InputState::Normal,
      running: true
    }
  }

  pub fn running(&self) -> bool {
    self.running
  }

  pub fn ui<B: Backend> (&self, f: &mut Frame<B>) {
    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints(
          [
              Constraint::Percentage(90),
              Constraint::Percentage(10),
          ]
          .as_ref(),
      )
      .split(f.size());
    
    // TODO: 
    //  - Figure out how to highlight text
    //  - Probably need some sort of styling component
    //  - Detect change in terminal size
    //  - Jump to line position by using function of terminal height and scroll offset
    //  - Better handle end of file

    // let height = Text::from(self.content.content()).inner().height();
    // let num_lines = text.lines.len();
    // let command = Paragraph::new(Text::from(height.to_string()));

    let text = Text::from(self.content.content());
    let file_content = Paragraph::new(text)
      .scroll((self.scroll_offset, 0));
    let command = Paragraph::new(Text::from(self.command_prompt.command()));
    let cursor_position = chunks[1].x + self.command_prompt.command().len() as u16;

    f.render_widget(file_content, chunks[0]);
    f.render_widget(command, chunks[1]);

    match self.state {
      InputState::Command =>  f.set_cursor(cursor_position, chunks[1].y),
      _ => ()
    }

  }

  pub fn parse_input(&mut self, key: KeyCode) {
    match self.state {
      InputState::Normal => self.parse_normal(key),
      InputState::Command => self.parse_command(key)
    }
  }

  fn parse_command(&mut self, key: KeyCode) {
    match key {
      KeyCode::Enter => {
        self.state = InputState::Normal;
        // run command 
        self.command_prompt.change_prompt(':')
      },
      _ => self.command_prompt.parse(key)
    }
  }

  fn parse_normal(&mut self, key:KeyCode) {
    match key {
      KeyCode::Up => self.scroll_up(),
      KeyCode::Down => self.scroll_down(),
      KeyCode::Char('q') => self.running = false,
      KeyCode::Char(c) if command_character(c) => {
        self.state = InputState::Command;
        self.command_prompt.change_prompt(c);
      },
      _ => ()
    }
  }

  fn scroll_down(&mut self) {
    let above_bottom = (self.scroll_offset + get_terminal_height()) <= self.content.num_lines() + SCREEN_END_OFFSET;
    if above_bottom {
      self.scroll_offset += 1
    }
  }

  fn scroll_up(&mut self) {
    if self.scroll_offset > 0 {
      self.scroll_offset -= 1;
    }
  }
}

fn get_terminal_height() -> u16 {
  let (_, terminal_height) = term_size::dimensions().unwrap();
  terminal_height as u16
}

fn command_character(character: char) -> bool {
  character == '/' || character == '?'
}
