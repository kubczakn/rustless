pub static USAGE: &str =
"
Missing filename (run with 'help' for help).
";

pub static HELP: &str = 
"
              SUMMARY OF RUSTLESS COMMANDS

Usage: cargo run -- [FILENAME]
---------------------------------------------------------------------------

                         MOVING

j  Down Arrow     *  Forward  one line.
k  Up Arrow       *  Backward one line.
f                 *  Forward  one window.
b                 *  Backward one window.
g                 *  Jump to start of file.
G                 *  Jump to end of file.
      ---------------------------------------------------
            'Window' represents the screen height.

---------------------------------------------------------------------------

                        SEARCHING

/pattern          *  Search and bold text that matches pattern. 
&pattern          *  Display only lines containing text matching pattern.
n                 *  Jump forward to next line with pattern.
N                 *  Jump backward to next line with pattern.
Up Arrow          *  Navigate back command history (must be in command mode).
Down Arrow        *  Navigate forward command history (must be in command mode).
      ---------------------------------------------------
       Command mode is started when '/' or '&' is entered.

---------------------------------------------------------------------------

                      CHANGING FILES

;e [file]            Examine a new file.
q                    Exit Rustless.

---------------------------------------------------------------------------
";