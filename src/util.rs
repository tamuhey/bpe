#[macro_export]
macro_rules! return_err {
    ($($arg:tt)*) => {
        log::error!($($arg)*);
        return Err(anyhow!($($arg)*));
    };
}
