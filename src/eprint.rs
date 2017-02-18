
macro_rules! eprint {
    ($($args:tt)*) => {{
        use std::io::Write;
        let _ = write!(::std::io::stderr(), $($args)*);
    }};
}

macro_rules! eprintln {
    ($($args:tt)*) => {{
        use std::io::Write;
        let _ = writeln!(::std::io::stderr(), $($args)*);
    }};
}

macro_rules! eprintln_bold {
    ($($args:tt)*) => {{
        use termion::{style};
        eprint!("{}", style::Bold);
        eprintln!( $($args)*);
        eprint!("{}", style::Reset);
    }};
}

macro_rules! eprintln_green {
    ($($args:tt)*) => {{
        use termion::{color};
        eprint!("{}", color::Fg(color::Green));
        eprintln!( $($args)*);
        eprint!("{}", color::Fg(color::Reset));
    }};
}

macro_rules! eprintln_red {
    ($($args:tt)*) => {{
        use termion::{color};
        eprint!("{}", color::Fg(color::Red));
        eprintln!( $($args)*);
        eprint!("{}", color::Fg(color::Reset));
    }};
}
