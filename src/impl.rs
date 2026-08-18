//! Built‑in error implementations.
//!
//! This module provides a ready‑to‑use [`IThrowable`] implementation – [`StringException`],
//! along with the [`throw_string!`] macro for convenient creation of such errors.

use alloc::boxed::Box;
use alloc::string::String;
use core::fmt::{Display, Formatter};
use crate::{Error, IThrowable};

/// A simple [`IThrowable`] implementation that stores a string message and an optional cause.
///
/// Usually created via [`StringException::new`] or the [`throw_string!`] macro.
pub struct StringException {
    message: String,
    cause: Option<Box<Error>>,
}

impl StringException {
    /// Creates a new `StringException` instance, boxed as `Box<dyn IThrowable>`.
    ///
    /// # Parameters
    /// - `message` – the error text.
    /// - `cause` – an optional cause (another error).
    ///
    /// # Example
    /// ```
    /// # use dyn_trace_err::{Error, r#impl::StringException};
    /// # use dyn_trace_err::IThrowable;
    /// let err = StringException::new("Oops".to_string(), None);
    /// assert_eq!(err.to_string(), "Oops");
    /// ```
    #[inline(always)]
    pub fn new(message: String, cause: Option<Error>) -> Box<dyn IThrowable> {
        Box::new(Self {
            message,
            cause: cause.map(Box::new),
        })
    }
}

impl IThrowable for StringException {
    #[inline(always)]
    fn cause(&self) -> &Option<Box<Error>> {
        &self.cause
    }
}

impl Display for StringException {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.message)
    }
}

/// Macro for quickly creating a [`StringException`] error and returning it immediately.
///
/// ## Available forms
///
/// ### When `!no-trace` (i.e. `all-trace` or `my-trace`)
/// - `throw_string!($msg)` – only a message.
/// - `throw_string!($msg, $cause)` – message and cause.
/// - `throw_string!($msg, $cause, $trace)` – message, cause, and explicit trace.
///
/// ### When `no-trace`
/// - `throw_string!($msg)` – only a message.
/// - `throw_string!($msg, $cause)` – message and cause.
///
/// # Example
/// ```
/// # use dyn_trace_err::{throw_string, Error};
/// # fn example() -> Result<(), Error> {
/// throw_string!("Invalid input");
/// # Ok(())
/// # }
/// ```
#[cfg(not(feature = "no-trace"))]
#[macro_export]
macro_rules! throw_string {
    ($msg:expr) => {
        $crate::throw!($crate::r#impl::StringException::new($msg.to_string(), None));
    };
    ($msg:expr, $cause:expr) => {
        $crate::throw!($crate::r#impl::StringException::new($msg.to_string(), $cause));
    };
    ($msg:expr, $cause:expr, $trace:expr) => {
        $crate::throw!($crate::r#impl::StringException::new($msg.to_string(), $cause), $trace);
    };
}

#[cfg(feature = "no-trace")]
#[macro_export]
macro_rules! throw_string {
    ($msg:expr) => {
        $crate::throw!($crate::r#impl::StringException::new($msg.to_string(), None));
    };
    ($msg:expr, $cause:expr) => {
        $crate::throw!($crate::r#impl::StringException::new($msg.to_string(), $cause));
    };
}