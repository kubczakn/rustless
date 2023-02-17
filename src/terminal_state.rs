use std::{collections::BTreeSet, cmp::min};
use std::ops::Bound::*;
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

use crate::pattern_history::PatternHistory;

const SCREEN_END_OFFSET: u16 = 10;

struct PatternState<'a> {
  pattern_history: PatternHistory,
  text: Text<'a>,
  next_pattern: BTreeSet<u16>
}

impl<'a> PatternState<'a> {
  fn new(content_in: &'a str) -> PatternState<'a> {
    PatternState {
      pattern_history: PatternHistory::new(),
      text: Text::from(content_in.clone()),
      next_pattern: BTreeSet::new()
    }
  }
}

pub struct TerminalState<'a> {
  pub running: bool,
  pub normal_mode: bool,
  scroll_offset: u16,
  num_lines: u16,
  content: &'a str,
  command: String,
  pattern_state: PatternState<'a>
}

impl<'a> TerminalState<'a> {
  pub fn new(num_lines_in: u16, content_in: &'a str) -> TerminalState<'a> {
    TerminalState {
      running: true,
      normal_mode: true,
      scroll_offset: 0,
      num_lines: num_lines_in, 
      content: content_in,
      command: String::from(":"),
      pattern_state: PatternState::new(content_in)
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

  let text = terminal_state.pattern_state.text.clone();
  let prompt = terminal_state.command.clone();

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
    KeyCode::Char('g') => terminal_state.scroll_offset = 0,
    KeyCode::Char('G') => terminal_state.scroll_offset = end_of_file_offset(terminal_state.num_lines), 
    KeyCode::Char('n') => {
      if let Some(next_match_scroll_offset) = scroll_to_next_match(terminal_state.scroll_offset, &terminal_state.pattern_state.next_pattern) {
        terminal_state.scroll_offset = next_match_scroll_offset
      }
    },
    KeyCode::Char('N') => {
      if let Some(previous_match_scroll_offset) = scroll_to_prior_match(terminal_state.scroll_offset, &terminal_state.pattern_state.next_pattern) {
        terminal_state.scroll_offset = previous_match_scroll_offset
      }
    },
    KeyCode::Char('b') => {
      terminal_state.scroll_offset = move_back_page(terminal_state.scroll_offset)
    },
    KeyCode::Char('B') => {
      terminal_state.scroll_offset = move_forward_page(terminal_state.num_lines, terminal_state.scroll_offset)
    },  
    KeyCode::Char(c) if command_character(c) => {
      terminal_state.normal_mode = false;
      terminal_state.command = String::from(c);
    },
    _ => ()
  };
  terminal_state
}

fn parse_command(key: KeyCode, mut terminal_state: TerminalState) -> TerminalState {
  match key {
    KeyCode::Char(c) => {
      terminal_state.command.push(c);
    },
    KeyCode::Backspace => {
      if terminal_state.command.len() == 1 {
        terminal_state = handle_normal_mode_transition(terminal_state);
      }
      else {
        terminal_state.command = pop_back_command(terminal_state.command);
      }
    },
    KeyCode::Enter => {
      terminal_state = update_pattern_state(terminal_state);
      terminal_state = handle_normal_mode_transition(terminal_state);
    },
    KeyCode::Up => {
      if let Some(prior_pattern) = terminal_state.pattern_state.pattern_history.get_prior_pattern() {
        terminal_state.command = String::from(terminal_state.command.chars().nth(0).unwrap()) + &prior_pattern;
      }
    }, 
    KeyCode::Down => {
      if let Some(next_pattern) = terminal_state.pattern_state.pattern_history.get_next_pattern() {
        terminal_state.command = String::from(terminal_state.command.chars().nth(0).unwrap()) + &next_pattern;
      }
    },
    _ => ()
  };
  terminal_state
}

fn handle_normal_mode_transition(mut terminal_state: TerminalState) -> TerminalState {
  terminal_state.command = String::from(":");
  terminal_state.normal_mode = true;
  terminal_state.pattern_state.pattern_history.reset_index();
  terminal_state
}

pub fn pop_back_command(mut command: String) -> String {
  if command.len() > 1 {
    command.pop();
  }
  command
}

fn end_of_file_offset(num_lines: u16) -> u16 {
  (num_lines + SCREEN_END_OFFSET) - get_terminal_height()
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

fn scroll_to_next_match(curr_offset: u16, match_line_numbers: &BTreeSet<u16>) -> Option<u16> {
  match_line_numbers.range((Excluded(curr_offset), Unbounded)).next().copied()
}

fn scroll_to_prior_match(curr_offset: u16, match_line_numbers: &BTreeSet<u16>) -> Option<u16> {
  match_line_numbers.range((Unbounded, Excluded(curr_offset))).next_back().copied()
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
  let split_regex = create_split_regex(pattern);

  let styled_line: Vec<Span> = split_regex.find_iter(line)
    .map(|elem| style_match(&pattern_regex, elem.as_str()))
    .collect();

  Spans::from(styled_line)
}

fn get_match_line_numbers(pattern: &str, content: &str) -> BTreeSet<u16> {
  let pattern_regex = Regex::new(pattern).unwrap();
  let mut matched_line_numbers: BTreeSet<u16> = BTreeSet::new();
  let mut line_number = 0;

  for line in content.lines() {
    if pattern_regex.is_match(line) {
      matched_line_numbers.insert(line_number);
      // println!("{}", line);
    }
    line_number += 1;
  }

  BTreeSet::from(matched_line_numbers)
}

fn get_matched_text<'a>(pattern: &str, content: &'a str) -> Text<'a> {
  Text::from(c![style_matches(pattern, line), for line in content.lines()])
}

fn update_pattern_state<'a>(mut terminal_state: TerminalState) -> TerminalState {
  terminal_state.pattern_state.text = get_matched_text(&terminal_state.command[1..], terminal_state.content);
  terminal_state.pattern_state.pattern_history.add_pattern(terminal_state.command[1..].to_string());
  terminal_state.pattern_state.next_pattern = get_match_line_numbers(&terminal_state.command[1..], terminal_state.content);

  terminal_state
}

fn create_split_regex(pattern: &str) -> Regex {
  if pattern.is_empty() {
    Regex::new(".").unwrap()
  }
  else {
    Regex::new(format!(r"{}|.", pattern).as_str()).unwrap()
  }
}

fn move_forward_page(num_lines: u16, scroll: u16) -> u16 {
  min(end_of_file_offset(num_lines), scroll + get_terminal_height())
}

fn move_back_page(scroll: u16) -> u16 {
  if let Some(subtracted_scroll) = scroll.checked_sub(get_terminal_height()) {
    subtracted_scroll
  }
  else {
    0
  }
}
