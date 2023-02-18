use std::{
  collections::BTreeSet, 
  cmp::min,
};
use std::ops::Bound::*;
use tui::{
  backend::{Backend},
  terminal::Frame,
  text::{Text},
  widgets::{Paragraph},
  layout::{Layout, Constraint, Direction, Rect},
};
use crossterm::{
  event::{KeyCode},
};

use crate::{
  text_state::TextState,
  command
};

const PAGE_HEIGHT_OFFSET: u16 = 1; // account for prompt 

pub struct TerminalState<'a> {
  pub running: bool,
  scroll_offset: u16,
  command: command::Command,
  text_state: TextState<'a>
}

impl<'a> TerminalState<'a> {
  pub fn new(content: String) -> TerminalState<'a> {
    TerminalState {
      running: true,
      scroll_offset: 0,
      command: command::Command::new(command::CommandCharacter::Normal),
      text_state: TextState::new(content)
    }
  }

  pub fn normal_mode(&self) -> bool {
    self.command.prompt == command::CommandCharacter::Normal
  }

  fn line_count(&self) -> usize {
    self.text_state.text.lines.len()
  }
}

pub fn parse_input(key: KeyCode, terminal_state: TerminalState) -> TerminalState {
  if terminal_state.normal_mode() {
    parse_normal(key, terminal_state)    
  }
  else {
    parse_command(key, terminal_state)
  }
}

pub fn ui<B: Backend> (f: &mut Frame<B>, terminal_state: &TerminalState) {
  let chunks = create_chunks(f);

  let line_start = min(terminal_state.scroll_offset as usize, terminal_state.line_count());
  let line_end = min((get_page_height() as usize) + line_start, terminal_state.line_count());

  let lines_to_display = Vec::from(&terminal_state.text_state.text.lines[line_start..line_end]);
  let text = Text::from(lines_to_display);
  let command_prompt = terminal_state.command.get_prompt();

  if !terminal_state.normal_mode() {
    let cursor_x_position = chunks[1].x + command_prompt.len() as u16;
    f.set_cursor(cursor_x_position, chunks[1].y);
  }

  let command = Paragraph::new(Text::from(command_prompt));
  let file_content = Paragraph::new(text);

  f.render_widget(file_content, chunks[0]);
  f.render_widget(command, chunks[1]);
}

fn parse_normal(key: KeyCode, mut terminal_state: TerminalState) -> TerminalState {
  match key {
    KeyCode::Up | KeyCode::Char('k') => terminal_state.scroll_offset = scroll_up(terminal_state.scroll_offset),
    KeyCode::Down | KeyCode::Char('j') => {
      terminal_state.scroll_offset = scroll_down(terminal_state.scroll_offset, terminal_state.line_count() as u16)
    },
    KeyCode::Char('q') => terminal_state.running = false,
    KeyCode::Char('g') => terminal_state.scroll_offset = 0,
    KeyCode::Char('G') => terminal_state.scroll_offset = end_of_file_offset(terminal_state.line_count() as u16), 
    KeyCode::Char('n') => {
      if let Some(next_match_scroll_offset) = scroll_to_next_match(terminal_state.scroll_offset, &terminal_state.text_state.next_pattern) {
        terminal_state.scroll_offset = next_match_scroll_offset
      }
    },
    KeyCode::Char('N') => {
      if let Some(previous_match_scroll_offset) = scroll_to_prior_match(terminal_state.scroll_offset, &terminal_state.text_state.next_pattern) {
        terminal_state.scroll_offset = previous_match_scroll_offset
      }
    },
    KeyCode::Char('b') => {
      terminal_state.scroll_offset = move_back_page(terminal_state.scroll_offset)
    },
    KeyCode::Char('B') => {
      terminal_state.scroll_offset = move_forward_page(terminal_state.line_count() as u16, terminal_state.scroll_offset)
    },  
    KeyCode::Char(c) => {
      if let Some(command_character) = command::CommandCharacter::command_character(c) {
        terminal_state.command = command::Command::new(command_character);
      }
    },
    _ => ()
  };
  terminal_state
}

fn parse_command<'a>(key: KeyCode, mut terminal_state: TerminalState<'a>) -> TerminalState<'a> {
  match key {
    KeyCode::Char(c) => {
      terminal_state.command.command_text.push(c);
    },
    KeyCode::Backspace => {
      if terminal_state.command.command_text.is_empty() {
        terminal_state = handle_normal_mode_transition(terminal_state);
      }
      else {
        terminal_state.command.command_text.pop();
      }
    },
    KeyCode::Enter => {
      terminal_state = update_text_state(terminal_state);
      terminal_state = handle_normal_mode_transition(terminal_state);
    },
    KeyCode::Up => {
      if let Some(prior_pattern) = terminal_state.text_state.pattern_history.get_prior_pattern() {
        terminal_state.command.command_text = String::from(&prior_pattern);
      }
    }, 
    KeyCode::Down => {
      if let Some(next_pattern) = terminal_state.text_state.pattern_history.get_next_pattern() {
        terminal_state.command.command_text = String::from(&next_pattern);
      }
    },
    _ => ()
  };
  terminal_state
}

fn handle_normal_mode_transition(mut terminal_state: TerminalState) -> TerminalState {
  terminal_state.command = command::Command::new(command::CommandCharacter::Normal);
  terminal_state.text_state.pattern_history.reset_index();
  terminal_state
}

fn end_of_file_offset(num_lines: u16) -> u16 {
  if let Some(end_of_file_offset) = num_lines.checked_sub(get_page_height()) {
    end_of_file_offset
  }
  else {
    0
  }
}

fn scroll_down(mut scroll: u16, num_lines: u16) -> u16 {
  let above_bottom = (scroll + 1) <= end_of_file_offset(num_lines);
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
            Constraint::Percentage(99),
            Constraint::Percentage(1),
        ]
        .as_ref(),
    )
    .split(f.size())
}

fn get_page_height() -> u16 {
  let (_, terminal_height) = term_size::dimensions().unwrap();
  (terminal_height as u16) - PAGE_HEIGHT_OFFSET
}

fn move_forward_page(num_lines: u16, scroll: u16) -> u16 {
  min(end_of_file_offset(num_lines), scroll + get_page_height())
}

fn move_back_page(scroll: u16) -> u16 {
  if let Some(subtracted_scroll) = scroll.checked_sub(get_page_height()) {
    subtracted_scroll
  }
  else {
    0
  }
}

fn update_text_state<'a>(mut terminal_state: TerminalState<'a>) -> TerminalState<'a> {
  match terminal_state.command.prompt {
    command::CommandCharacter::MatchLines => {
      terminal_state.text_state = terminal_state.text_state.match_lines(&terminal_state.command.command_text);
      terminal_state.scroll_offset = 0;
    },
    command::CommandCharacter::SearchForward | command::CommandCharacter::SearchBackwards => {
      terminal_state.text_state = terminal_state.text_state.perform_search(&terminal_state.command.command_text);
    },
    command::CommandCharacter::ChangeFile => {
      terminal_state.text_state = terminal_state.text_state.change_file(&terminal_state.command.command_text);
      terminal_state.scroll_offset = 0;
    },
    command::CommandCharacter::Normal => ()
  }

  terminal_state
}
