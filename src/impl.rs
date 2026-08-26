//! Built‑in error implementations.
//!
//! This module provides ready‑to‑use [`IThrowable`] implementations:
//! - [`StringException`] – stores a `String` message and an optional cause.
//! - [`DisplayableException`] – wraps any type that implements [`Display`].
//!
//! Also provides the [`throw_string!`] and [`throw_display!`] macros for convenient error creation.

use alloc::boxed::Box;
use alloc::string::String;
use core::fmt::{Display, Formatter};
use crate::{Error, IThrowable};

/// A simple [`IThrowable`] implementation that stores a string message and an optional cause.
///
/// Usually created via [`StringException::new`] or the [`throw_string!`] macro.
///
/// # Example
/// ```
/// # use dyn_trace_err::{Error, r#impl::StringException};
/// # use dyn_trace_err::IThrowable;
/// let err = StringException::new("Oops".to_string(), None);
/// assert_eq!(err.to_string(), "Oops");
/// ```
pub struct StringException {
    message: String,
    cause: Option<Box<Error<dyn IThrowable>>>,
}

/// A [`IThrowable`] implementation that wraps any type implementing [`Display`].
///
/// This is useful when you already have an error type that implements `Display`
/// but you don't want to implement `IThrowable` manually.
///
/// Usually created via [`DisplayableException::new`] or the [`throw_display!`] macro.
///
/// # Example
/// ```
/// # use dyn_trace_err::{Error, r#impl::DisplayableException};
/// # use std::fmt;
/// #[derive(Debug)]
/// enum MyError { Foo }
/// impl fmt::Display for MyError {
///     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "Foo error") }
/// }
/// let err = DisplayableException::new(Box::new(MyError::Foo), None);
/// assert_eq!(err.to_string(), "Foo error");
/// ```
pub struct DisplayableException {
    display: Box<dyn Display>,
    cause: Option<Box<Error<dyn IThrowable>>>,
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
    pub fn new(message: String, cause: Option<Error<dyn IThrowable>>) -> Box<dyn IThrowable> {
        Box::new(Self {
            message,
            cause: cause.map(Box::new),
        })
    }
}

impl IThrowable for StringException {
    #[inline(always)]
    fn cause(&self) -> &Option<Box<Error<dyn IThrowable>>> {
        &self.cause
    }
}

impl Display for StringException {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.message)
    }
}

impl DisplayableException {
    /// Creates a new `DisplayableException` instance, boxed as `Box<dyn IThrowable>`.
    ///
    /// # Parameters
    /// - `display` – a boxed value that implements `Display`.
    /// - `cause` – an optional cause (another error).
    ///
    /// # Example
    /// ```
    /// # use dyn_trace_err::{Error, r#impl::DisplayableException};
    /// # use std::fmt;
    /// # #[derive(Debug)]
    /// # enum MyError { Foo }
    /// # impl fmt::Display for MyError {
    /// #     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "Foo error") }
    /// # }
    /// let err = DisplayableException::new(Box::new(MyError::Foo), None);
    /// assert_eq!(err.to_string(), "Foo error");
    /// ```
    #[inline(always)]
    pub fn new(display: Box<dyn Display>, cause: Option<Error<dyn IThrowable>>) -> Box<dyn IThrowable> {
        Box::new(Self {
            display,
            cause: cause.map(Box::new),
        })
    }
}

impl IThrowable for DisplayableException {
    #[inline(always)]
    fn cause(&self) -> &Option<Box<Error<dyn IThrowable>>> {
        &self.cause
    }
}

impl Display for DisplayableException {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.display.fmt(f)
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
/// # use dyn_trace_err::{throw_string, Error, IThrowable};
/// # fn example() -> Result<(), Error<dyn IThrowable>> {
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

/// Macro for quickly creating a [`DisplayableException`] error and returning it immediately.
///
/// ## Available forms
///
/// ### When `!no-trace` (i.e. `all-trace` or `my-trace`)
/// - `throw_display!($expr)` – only a displayable value.
/// - `throw_display!($expr, $cause)` – displayable value and cause.
/// - `throw_display!($expr, $cause, $trace)` – displayable value, cause, and explicit trace.
///
/// ### When `no-trace`
/// - `throw_display!($expr)` – only a displayable value.
/// - `throw_display!($expr, $cause)` – displayable value and cause.
///
/// # Example
/// ```
/// # use dyn_trace_err::{throw_display, IThrowable};
/// # use std::fmt;
/// # #[derive(Debug)] enum E { A }
/// # impl fmt::Display for E { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "E") } }
/// # fn example() -> Result<(), dyn_trace_err::Error<dyn IThrowable>> {
/// throw_display!(E::A);
/// # Ok(())
/// # }
/// ```
#[cfg(not(feature = "no-trace"))]
#[macro_export]
macro_rules! throw_display {
    ($display:expr) => {
        $crate::throw!($crate::r#impl::DisplayableException::new($crate::Box::new($display), None));
    };
    ($display:expr, $cause:expr) => {
        $crate::throw!($crate::r#impl::DisplayableException::new($crate::Box::new($display), $cause));
    };
    ($display:expr, $cause:expr, $trace:expr) => {
        $crate::throw!($crate::r#impl::DisplayableException::new($crate::Box::new($display), $cause), $trace);
    };
}

#[cfg(feature = "no-trace")]
#[macro_export]
macro_rules! throw_display {
    ($display:expr) => {
        $crate::throw!($crate::r#impl::DisplayableException::new($crate::Box::new($display), None));
    };
    ($display:expr, $cause:expr) => {
        $crate::throw!($crate::r#impl::DisplayableException::new($crate::Box::new($display), $cause));
    };
}