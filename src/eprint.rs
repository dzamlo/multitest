macro_rules! eprint {
    ($($args:tt)*) => {
        use std::io::Write;
        let _ = write!(::std::io::stderr(), $($args)*);
    };
}

macro_rules! eprintln {
    ($($args:tt)*) => {
        use std::io::Write;
        let _ = writeln!(::std::io::stderr(), $($args)*);
    };
}
