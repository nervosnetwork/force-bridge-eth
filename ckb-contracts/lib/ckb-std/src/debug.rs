/// debug macro
///
/// Output a debug message.
/// This macro only compiled under debug build and does nothing in release build.
///
/// Notice: to see the debug output, you must turn on `ckb_script` debugging log in the CKB node configuration.
///
/// # Example
///
/// ```
/// debug!("hello world");
/// debug!("there is a universal error caused by {}", 42);
/// ```
#[macro_export]
macro_rules! debug {
    ($fmt:literal) => {
        #[cfg(debug_assertions)]
        $crate::syscalls::debug(alloc::format!($fmt));
    };
    ($fmt:literal, $($args:expr),+) => {
        #[cfg(debug_assertions)]
        $crate::syscalls::debug(alloc::format!($fmt, $($args), +));
    };
}
