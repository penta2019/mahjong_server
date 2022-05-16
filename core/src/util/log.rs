#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!(
            "[ERROR]({}:{}) {}",
            file!(),
            line!(),
            format_args!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        eprintln!(
            "[WARN]({}:{}) {}",
            file!(),
            line!(),
            format_args!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        eprintln!(
            "[INFO]({}:{}) {}",
            file!(),
            line!(),
            format_args!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        eprintln!(
            "[DEBUG]({}:{}) {}",
            file!(),
            line!(),
            format_args!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! error_exit {
    ($($arg:tt)*) => {{
        eprintln!(
            "[ERROR]({}:{}) {}",
            file!(),
            line!(),
            format_args!($($arg)*)
        );
        std::process::exit(1);
    }};
}
