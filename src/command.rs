#[derive(PartialEq)]
pub enum CommandCharacter {
  Normal,
  Search,
  MatchLines,
  ChangeFile
}

impl CommandCharacter {
  pub fn as_char(&self) -> char {
    match self {
      CommandCharacter::Normal => ':',
      CommandCharacter::Search => '/',
      CommandCharacter::MatchLines => '&',
      CommandCharacter::ChangeFile => ';'
    }
  }

  pub fn command_character(character: char) -> Option<CommandCharacter> {
    match character {
      ':' => Some(CommandCharacter::Normal),
      '/' => Some(CommandCharacter::Search),
      '&' => Some(CommandCharacter::MatchLines),
      ';' => Some(CommandCharacter::ChangeFile),
      _ => None
    }
  }
}

pub struct Command {
  pub prompt: CommandCharacter,
  pub command_text: String
}

impl Command {
  pub fn new(prompt_in: CommandCharacter) -> Command {
    Command {
      prompt: prompt_in,
      command_text: String::new()
    }
  }

  pub fn get_prompt(&self) -> String {
    String::from(self.prompt.as_char()) + &self.command_text
  }
}


