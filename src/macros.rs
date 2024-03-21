#[cfg(feature = "debug")]
macro_rules! debug {
    ($($arg:tt)*) => {
        print!("[{:>w$}] \t", module_path!(), w = 28);
        println!($($arg)*)
    }
}

#[cfg(not(feature = "debug"))]
macro_rules! debug {
    ($($arg:tt)*) => {};
}
