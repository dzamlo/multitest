use termcolor::ColorChoice;

static mut COLOR_CHOICE: ColorChoice = ColorChoice::Auto;


/// This function modify a `static mut` variable. It is unsafe to use after any thread is launched.
pub unsafe fn set_color_choice(new_color_choice: ColorChoice) {
    COLOR_CHOICE = new_color_choice;
}

pub fn color_choice() -> ColorChoice {
    // This function is safe if set_color_choice is never used when there is multiple thread
    unsafe { COLOR_CHOICE }
}


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

macro_rules! eprintln_color {
    ($color_spec:expr, $($args:tt)*) => {{
        use std::io::Write;
        use termcolor::{StandardStream, WriteColor};
        let mut stderr = StandardStream::stderr(::eprint::color_choice());
        let _ = stderr.set_color($color_spec);
        writeln!(stderr, $($args)*).unwrap();
        let _ = stderr.reset();
    }};
}

macro_rules! eprintln_bold {
    ($($args:tt)*) => {{
        use termcolor::ColorSpec;
        let mut color_spec = ColorSpec::new();
        color_spec.set_bold(true);
        eprintln_color!(&color_spec, $($args)*);
    }};
}

macro_rules! eprintln_green {
    ($($args:tt)*) => {{
        use termcolor::{Color, ColorSpec};
        let mut color_spec = ColorSpec::new();
        color_spec.set_fg(Some(Color::Green));
        eprintln_color!(&color_spec, $($args)*);
    }};
}

macro_rules! eprintln_red {
    ($($args:tt)*) => {{
        use termcolor::{Color, ColorSpec};
        let mut color_spec = ColorSpec::new();
        color_spec.set_fg(Some(Color::Red));
        eprintln_color!(&color_spec, $($args)*);
    }};
}
