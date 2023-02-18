use std::{collections::VecDeque, cmp::{min}};

pub struct PatternHistory {
  current_pattern_index: usize,
  prior_patterns: VecDeque<String>
}

impl PatternHistory {
  pub fn new() -> PatternHistory {
    PatternHistory {
      current_pattern_index: 0,
      prior_patterns: VecDeque::from([String::from("")]) // used as sentinel value
    }
  }

  pub fn reset_index(&mut self) {
    self.current_pattern_index = 0
  }

  pub fn add_pattern(mut self, pattern_in: String) -> PatternHistory {
    self.prior_patterns.pop_front();
    self.prior_patterns.push_front(pattern_in);
    self.prior_patterns.push_front(String::from(""));

    self
  }

  pub fn get_prior_pattern(&mut self) -> Option<String> {
    if self.is_empty() {
      None
    }
    else {
      self.move_index_back_history();
      let result = self.prior_patterns.get(self.current_pattern_index).unwrap().clone();
      Some(result)
    }
  }

  pub fn get_next_pattern(&mut self) -> Option<String> {
    let index_at_most_recent_pattern_or_sentinel = self.current_pattern_index <= 1;
    if self.is_empty() || index_at_most_recent_pattern_or_sentinel {
      None
    }
    else {
      self.move_index_forward_history();
      let result = self.prior_patterns.get(self.current_pattern_index).unwrap().clone();
      Some(result)
    }
  }

  fn is_empty(&self) -> bool {
    self.prior_patterns.len() == 1
  }

  fn move_index_back_history(&mut self) {
    self.current_pattern_index = min(self.current_pattern_index + 1, self.prior_patterns.len() - 1)
  }

  fn move_index_forward_history(&mut self) {
    if let Some(decremented_pattern_index) = self.current_pattern_index.checked_sub(1) {
      self.current_pattern_index = decremented_pattern_index;
    }
  }
}
