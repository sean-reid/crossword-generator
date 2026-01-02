#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        #[cfg(feature = "debug")]
        {
            web_sys::console::log_1(&format!($($arg)*).into());
        }
    };
}
