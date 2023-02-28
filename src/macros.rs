//! Minimal versions of standard-library panicking and printing macros.
//!
//! We're avoiding static initializers, so we can't have things like string
//! literals. Replace the standard assert macros with simpler implementations.

#[cfg(not(feature = "command"))]
pub(crate) fn puts(message: &[u8]) {
    let ctx = byte_array::str!("stderr");
    crate::bindings::wasi_console::log(crate::bindings::wasi_console::Level::Info, &ctx, &message);
}

#[cfg(feature = "command")]
pub(crate) fn puts(message: &[u8]) {
    core::arch::wasm32::unreachable()
}

/// A minimal `eprint` for debugging.
#[allow(unused_macros)]
macro_rules! eprint {
    ($arg:tt) => {{
        // We have to expand string literals into byte arrays to prevent them
        // from getting statically initialized.
        let message = byte_array::str!($arg);
        crate::macros::puts(&message)
    }};
}

/// A minimal `eprintln` for debugging.
#[allow(unused_macros)]
macro_rules! eprintln {
    ($arg:tt) => {{
        // We have to expand string literals into byte arrays to prevent them
        // from getting statically initialized.
        let message = byte_array::str_nl!($arg);
        let ctx = byte_array::str!("stderr");
        crate::macros::puts(&message)
    }};
}

pub(crate) fn eprint_u32(x: u32) {
    if x == 0 {
        eprint!("0");
    } else {
        eprint_u32_impl(x)
    }

    fn eprint_u32_impl(x: u32) {
        if x != 0 {
            eprint_u32_impl(x / 10);

            let digit = [b'0' + ((x % 10) as u8)];
            crate::macros::puts(&digit)
        }
    }
}

/// A minimal `unreachable`.
macro_rules! unreachable {
    () => {{
        eprint!("unreachable executed at line ");
        crate::macros::eprint_u32(line!());
        core::arch::wasm32::unreachable()
    }};

    ($arg:tt) => {{
        eprint!("unreachable executed at line ");
        crate::macros::eprint_u32(line!());
        eprint!(": ");
        eprintln!($arg);
        core::arch::wasm32::unreachable()
    }};
}

/// A minimal `assert`.
macro_rules! assert {
    ($cond:expr $(,)?) => {
        if !$cond {
            unreachable!("assertion failed")
        }
    };
}

/// A minimal `assert_eq`.
macro_rules! assert_eq {
    ($left:expr, $right:expr $(,)?) => {
        assert!($left == $right);
    };
}
