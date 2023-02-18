use std::{collections::BTreeSet, fs};
use cute::c;
use tui::{
  text::{Text, Span, Spans},
  style::{Style, Modifier, Color},
};
use regex::{Regex};

use crate::{
  pattern_history::{PatternHistory},
};

pub struct TextState<'a> {
  pub pattern_history: PatternHistory,
  pub next_pattern: BTreeSet<u16>,
  pub text: Text<'a>,
  pub content: String,
}

impl<'a> TextState<'a> {
  pub fn new(content_in: String) -> TextState<'a> {
    TextState {
      pattern_history: PatternHistory::new(),
      next_pattern: BTreeSet::new(),
      text: Text::from(content_in.clone()),
      content: content_in,
    }
  }

  pub fn match_lines(self, pattern: &str) -> TextState<'a> {
    TextState {
      pattern_history: self.pattern_history.add_pattern(pattern.to_string()),
      next_pattern: get_match_line_numbers(&self.content, pattern),
      text: get_match_lines(self.content.clone(), pattern),
      content: self.content,
    }
  }

  pub fn perform_search(self, pattern: &str) -> TextState<'a> {
    TextState {
      pattern_history: self.pattern_history.add_pattern(pattern.to_string()),
      next_pattern: get_match_line_numbers(&self.content, pattern),
      text: get_bolded_match_text(self.content.clone(), pattern),
      content: self.content,
    }
  }

  pub fn change_file(self, file_path: &str) -> TextState<'a> {
    let open_file_result = fs::read_to_string(file_path);
    match open_file_result {
      Ok(content) => {
        TextState {
          pattern_history: self.pattern_history,
          next_pattern: self.next_pattern,
          text: Text::from(content.clone()),
          content: content, 
        }
      },
      Err(_) => self
    }
  }
}

fn get_match_line_numbers(content: &str, pattern: &str) -> BTreeSet<u16> {
  let pattern_regex = Regex::new(pattern).unwrap();
  let mut matched_line_numbers: BTreeSet<u16> = BTreeSet::new();
  let mut line_number = 0;

  for line in content.lines() {
    if pattern_regex.is_match(line) {
      matched_line_numbers.insert(line_number);
    }
    line_number += 1;
  }

  BTreeSet::from(matched_line_numbers)
}

fn get_match_lines<'a>(content: String, pattern: &str) -> Text<'a> {
  let pattern_regex = Regex::new(pattern).unwrap(); 
  Text::from(content.lines()
    .filter(|line| pattern_regex.is_match(line))
    .map(|line| Spans::from(line.to_owned()))
    .collect::<Vec<_>>())
}

fn get_bolded_match_text<'a>(content: String, pattern: &str) -> Text<'a> {
  Text::from(c![style_matches(pattern, line), for line in content.lines().map(|line| line.to_owned())])
}

fn style_match<'a>(re: &Regex, text: String) -> Span<'a> {
  if re.is_match(&text) {
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

fn style_matches<'a>(pattern: &str, line: String) -> Spans<'a> {
  let pattern_regex = Regex::new(pattern).unwrap();
  let split_regex = create_split_regex(pattern);

  let styled_line: Vec<Span> = split_regex.find_iter(&line)
    .map(|elem| style_match(&pattern_regex, elem.as_str().to_owned()))
    .collect();

  Spans::from(styled_line)
}

fn create_split_regex(pattern: &str) -> Regex {
  if pattern.is_empty() {
    Regex::new(".").unwrap()
  }
  else {
    Regex::new(format!(r"{}|.", pattern).as_str()).unwrap()
  }
}