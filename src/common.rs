#[macro_export]
macro_rules! exit_with_error {
    ($($err_msg:tt)*) => {{
        eprintln!($($err_msg)*);
        std::process::exit(1);
    }}
}
