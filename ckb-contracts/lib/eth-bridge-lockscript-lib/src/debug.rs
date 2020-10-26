#[macro_export]
macro_rules! debug {
    ($fmt:literal) => {
        #[cfg(feature = "std")]
        dbg!(format!($fmt));
        #[cfg(not(feature = "std"))]
        ckb_std::syscalls::debug(alloc::format!($fmt));
    };
    ($fmt:literal, $($args:expr),+) => {
        #[cfg(feature = "std")]
        dbg!(format!($fmt, $($args), +));
        #[cfg(not(feature = "std"))]
        ckb_std::syscalls::debug(alloc::format!($fmt, $($args), +));
    };
}
