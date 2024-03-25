#[macro_export]
macro_rules! log {
    ($fmt:literal) => {
        #[cfg(feature = "log")]
        ckb_std::syscalls::debug(alloc::format!($fmt));
    };
    ($fmt:literal, $($args:expr),+) => {
        #[cfg(feature = "log")]
        ckb_std::syscalls::debug(alloc::format!($fmt, $($args), +));
        #[cfg(not(feature = "log"))]
        let _ = &($($args),+);
    };
}
