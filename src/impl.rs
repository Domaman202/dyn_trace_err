//! Built‑in error implementations.
//!
//! This module provides ready‑to‑use [`IThrowable`] implementations:
//! - [`StringException`] – stores a `String` message and an optional cause.
//! - [`FormattableException`] – wraps any type that implements [`Display`] + [`Debug`].
//!
//! Also provides the [`throw_string!`] and [`throw_formattable!`] macros for convenient error creation.

use alloc::boxed::Box;
use alloc::string::String;
use core::fmt::{Debug, Display, Formatter};
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

/// Trait for types that can be used with [`FormattableException`].
/// This is a marker trait that requires both [`Display`] and [`Debug`].
pub trait Formattable : Display + Debug {}

/// Wrapper for a [`Display`] type, used to make it [`Formattable`] (uses `Display` for both).
pub struct FormattableFromDisplay<T>(T) where T: Display;

/// Wrapper for a [`Debug`] type, used to make it [`Formattable`] (uses `Debug` for both).
pub struct FormattableFromDebug<T>(T) where T: Debug;

/// A [`IThrowable`] implementation that wraps any type implementing [`Formattable`].
///
/// This allows you to have different representations for `Display` (user‑friendly)
/// and `Debug` (detailed) for your error type.
///
/// Usually created via [`FormattableException::new`] or the [`throw_formattable!`] macro.
pub struct FormattableException {
    value: Box<dyn Formattable>,
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

impl Debug for StringException {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.message)
    }
}

impl<T> Display for FormattableFromDisplay<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Debug for FormattableFromDisplay<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Formattable for FormattableFromDisplay<T> where T: Display {}

impl<T> Display for FormattableFromDebug<T> where T: Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Debug for FormattableFromDebug<T> where T: Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl FormattableException {
    /// Creates a new `FormattableException` instance, boxed as `Box<dyn IThrowable>`.
    ///
    /// # Parameters
    /// - `value` – a boxed value that implements [`Formattable`] (i.e., `Display + Debug`).
    /// - `cause` – an optional cause (another error).
    ///
    /// # Example
    /// ```
    /// # use dyn_trace_err::{Error, r#impl::{FormattableException, Formattable}};
    /// # use std::fmt;
    /// # #[derive(Debug)]
    /// # struct MyError;
    /// # impl fmt::Display for MyError { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "display") } }
    /// # impl Formattable for MyError {}
    /// let err = FormattableException::new(Box::new(MyError), None);
    /// assert_eq!(err.to_string(), "display");
    /// ```
    #[inline(always)]
    pub fn new(value: Box<dyn Formattable>, cause: Option<Error<dyn IThrowable>>) -> Box<dyn IThrowable> {
        Box::new(Self {
            value,
            cause: cause.map(Box::new),
        })
    }
}

impl Display for FormattableException {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.value, f)
    }
}

impl Debug for FormattableException {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.value, f)
    }
}

impl IThrowable for FormattableException {
    fn cause(&self) -> &Option<Box<Error<dyn IThrowable>>> {
        &self.cause
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

/// Macro for quickly creating a [`FormattableException`] error and returning it immediately.
///
/// ## Available forms
///
/// ### When `!no-trace` (i.e. `all-trace` or `my-trace`)
/// - `throw_formattable!($expr)` – only a value.
/// - `throw_formattable!($expr, $cause)` – value and cause.
/// - `throw_formattable!($expr, $cause, $trace)` – value, cause, and explicit trace.
///
/// ### When `no-trace`
/// - `throw_formattable!($expr)` – only a value.
/// - `throw_formattable!($expr, $cause)` – value and cause.
///
/// # Example
/// ```
/// # use dyn_trace_err::{throw_formattable, IThrowable};
/// # use std::fmt;
/// # #[derive(Debug)] enum E { A }
/// # impl fmt::Display for E { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "E") } }
/// # impl dyn_trace_err::r#impl::Formattable for E {}
/// # fn example() -> Result<(), dyn_trace_err::Error<dyn IThrowable>> {
/// throw_formattable!(E::A);
/// # Ok(())
/// # }
/// ```
#[cfg(not(feature = "no-trace"))]
#[macro_export]
macro_rules! throw_formattable {
    ($display:expr) => {
        $crate::throw!($crate::r#impl::FormattableException::new($crate::Box::new($display), None));
    };
    ($display:expr, $cause:expr) => {
        $crate::throw!($crate::r#impl::FormattableException::new($crate::Box::new($display), $cause));
    };
    ($display:expr, $cause:expr, $trace:expr) => {
        $crate::throw!($crate::r#impl::FormattableException::new($crate::Box::new($display), $cause), $trace);
    };
}

#[cfg(feature = "no-trace")]
#[macro_export]
macro_rules! throw_formattable {
    ($display:expr) => {
        $crate::throw!($crate::r#impl::FormattableException::new($crate::Box::new($display), None));
    };
    ($display:expr, $cause:expr) => {
        $crate::throw!($crate::r#impl::FormattableException::new($crate::Box::new($display), $cause));
    };
}