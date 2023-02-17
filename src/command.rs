use std::{collections::BTreeSet};
use tui::{
  text::{Text, Span, Spans},
  style::{Style, Modifier, Color},
};
use regex::{Regex};
use cute::c;

pub struct Command {
  pub prompt: char,
  pub command_text: String
}

impl Command {
  pub fn new(prompt_in: char) -> Command {
    Command {
      prompt: prompt_in,
      command_text: String::new()
    }
  }

  pub fn get_prompt(&self) -> String {
    String::from(self.prompt) + &self.command_text
  }
}

pub fn pop_back_command(mut command: String) -> String {
  if command.len() > 1 {
    command.pop();
  }
  command
}

pub fn get_match_line_numbers(pattern: &str, content: &str) -> BTreeSet<u16> {
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

pub fn get_bolded_match_text<'a>(pattern: &str, content: &'a str) -> Text<'a> {
  Text::from(c![style_matches(pattern, line), for line in content.lines()])
}

pub fn get_match_lines<'a>(pattern: &str, content: &'a str) -> Text<'a> {
  let pattern_regex = Regex::new(pattern).unwrap(); 
  Text::from(content.lines()
    .filter(|line| pattern_regex.is_match(line))
    .map(|line| Spans::from(line))
    .collect::<Vec<_>>())
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

fn create_split_regex(pattern: &str) -> Regex {
  if pattern.is_empty() {
    Regex::new(".").unwrap()
  }
  else {
    Regex::new(format!(r"{}|.", pattern).as_str()).unwrap()
  }
}