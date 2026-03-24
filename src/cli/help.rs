use clap::builder::styling::{Ansi256Color, Color, Style, Styles};

pub const BANNER: &str = "

      \x1b[38;5;203m╭──────────────╮
      \x1b[38;5;203m│ \x1b[38;5;215m◉\x1b[38;5;203m   \x1b[38;5;117m┌───┐\x1b[38;5;203m \x1b[38;5;215mfj\x1b[0;m \x1b[38;5;203m│
      \x1b[38;5;203m│     \x1b[38;5;117m│   │\x1b[38;5;203m    │
      \x1b[38;5;203m│     \x1b[38;5;117m└───┘\x1b[38;5;203m    │
      \x1b[38;5;203m╰──────────────╯
           \x1b[38;5;203mf\x1b[38;5;215mu\x1b[38;5;226mj\x1b[38;5;154mi\x1b[38;5;117mn\x1b[38;5;135mx\x1b[0m";

const ARG: Style = Style::new()
    .bold()
    .fg_color(Some(Color::Ansi256(Ansi256Color(117))));

const HEADING: Style = Style::new()
    .bold()
    .underline()
    .fg_color(Some(Color::Ansi256(Ansi256Color(203))));

pub const STYLES: Styles = Styles::styled()
    .header(HEADING)
    .usage(HEADING)
    .literal(ARG)
    .placeholder(ARG);
