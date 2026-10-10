#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("debug.log")
        {
            let _ = writeln!(file, $($arg)*);
        }
    };
}
