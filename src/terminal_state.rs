use tui::{
  backend::{Backend},
  terminal::Frame,
  text::{Text, Span, Spans},
  style::{Style, Modifier, Color},
  widgets::{Paragraph},
  layout::{Layout, Constraint, Direction, Rect},
};
use crossterm::{
  event::{KeyCode},
};
use regex::{Regex};
use cute::c;

const SCREEN_END_OFFSET: u16 = 10;

pub struct TerminalState<'a> {
  pub running: bool,
  pub normal_mode: bool,
  scroll_offset: u16,
  num_lines: u16,
  content: &'a str,
  prompt: String,
  text: Text<'a>
}

impl<'a> TerminalState<'a> {
  pub fn new(num_lines_in: u16, content_in: &'a str) -> TerminalState<'a> {
    TerminalState {
      running: true,
      normal_mode: true,
      scroll_offset: 0,
      num_lines: num_lines_in,
      content: content_in,
      prompt: String::from(":"),
      text: Text::from(content_in.clone())
    }
  }
}

pub fn parse_input(key: KeyCode, terminal_state: TerminalState) -> TerminalState {
  if terminal_state.normal_mode {
    parse_normal(key, terminal_state)    
  }
  else {
    parse_command(key, terminal_state)
  }
}

pub fn ui<B: Backend> (f: &mut Frame<B>, terminal_state: &TerminalState) {
  let chunks = create_chunks(f);

  let text = terminal_state.text.clone();
  let prompt = terminal_state.prompt.clone();

  if !terminal_state.normal_mode {
    let cursor_x_position = chunks[1].x + prompt.len() as u16;
    f.set_cursor(cursor_x_position, chunks[1].y);
  }

  let command = Paragraph::new(Text::from(prompt));
  let file_content = Paragraph::new(text)
    .scroll((terminal_state.scroll_offset, 0));

  f.render_widget(file_content, chunks[0]);
  f.render_widget(command, chunks[1]);
}

fn parse_normal(key: KeyCode, mut terminal_state: TerminalState) -> TerminalState {
  match key {
    KeyCode::Up => terminal_state.scroll_offset = scroll_up(terminal_state.scroll_offset),
    KeyCode::Down => terminal_state.scroll_offset = scroll_down(terminal_state.scroll_offset, terminal_state.num_lines),
    KeyCode::Char('q') => terminal_state.running = false,
    KeyCode::Char(c) if command_character(c) => {
      terminal_state.normal_mode = false;
      terminal_state.prompt = String::from(c);
    },
    _ => ()
  };
  terminal_state
}

fn parse_command(key: KeyCode, mut terminal_state: TerminalState) -> TerminalState {
  match key {
    KeyCode::Char(c) => {
      terminal_state.prompt.push(c);
    },
    KeyCode::Backspace => {
      terminal_state.prompt = pop_back(terminal_state.prompt);
    },
    KeyCode::Enter => {
      terminal_state.normal_mode = true;
      terminal_state.text = get_matched_text(&terminal_state.prompt[1..], terminal_state.content);
      terminal_state.prompt = String::from(":");
      return terminal_state;
    }
    _ => ()
  };
  terminal_state
}

pub fn pop_back(mut command: String) -> String {
  if command.len() > 1 {
    command.pop();
  }
  command
}

fn scroll_down(mut scroll: u16, num_lines: u16) -> u16 {
  let above_bottom = (scroll + get_terminal_height()) <= num_lines + SCREEN_END_OFFSET;
  if above_bottom {
    scroll += 1
  }
  scroll
}

fn scroll_up(mut scroll: u16) -> u16 {
  if scroll > 0 {
    scroll -= 1;
  }
  scroll
}

fn create_chunks<B: Backend>(f: &Frame<B>) -> Vec<Rect> {
  Layout::default()
    .direction(Direction::Vertical)
    .constraints(
        [
            Constraint::Percentage(90),
            Constraint::Percentage(10),
        ]
        .as_ref(),
    )
    .split(f.size())
}

fn get_terminal_height() -> u16 {
  let (_, terminal_height) = term_size::dimensions().unwrap();
  terminal_height as u16
}

fn command_character(character: char) -> bool {
  character == '/' || character == '?'
}

fn style_match<'a>(re: &Regex, text: &'a str) -> Span<'a> {
  if re.is_match(text) {
    Span::styled(
      text, 
      Style::default()
        .add_modifier(Modifier::BOLD)
        .fg(Color::Red)
    )
  }
  else {
    Span::from(text)
  }
}

fn style_matches<'a>(pattern: &str, line: &'a str) -> Spans<'a> {
  let pattern_regex = Regex::new(pattern).unwrap();
  let split_regex = Regex::new(format!(r"{}|.", pattern).as_str()).unwrap();

  let styled_line: Vec<Span> = split_regex.find_iter(line)
    .map(|elem| style_match(&pattern_regex, elem.as_str()))
    .collect();

  Spans::from(styled_line)
}

fn get_matched_text<'a>(pattern: &str, content: &'a str) -> Text<'a> {
  Text::from(c![style_matches(pattern, line), for line in content.lines()])
}
