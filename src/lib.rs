//! # dyn_trace_err
//!
//! A flexible error‑handling library with stack trace support.
//!
//! ## Features
//! - Works in `no_std` (requires `alloc`).
//! - Three tracing modes, selectable via Cargo features:
//!   - `all-trace` — automatically adds `file!()` and `line!()`.
//!   - `my-trace` — you explicitly pass a `Trace` object.
//!   - `no-trace` — tracing completely disabled.
//! - Nested causes via the `IThrowable` trait.
//! - Convenient macros: `throw!`, `catch!`, `throw_string!`, `throw_formattable!`,
//!   and `throw_formattable_string!`.
//! - Built‑in error types: `StringException`, `FormattableException`, and
//!   `FormattableStringException`.
//! - Access to error payload via `throwable()` method.
//!
//! ## Formatting
//! The [`Error`] type implements two formatting traits:
//! - `Display` → prints **only the error message** (without causes or traces).
//! - `Debug`   → prints the **full error chain** with all causes and stack traces.
//!
//! This separation allows you to show concise user‑friendly messages or detailed
//! diagnostic information depending on the context.
//!
//! ## Example
//! ```
//! use dyn_trace_err::{throw_string, catch, Error, IThrowable};
//!
//! fn bar(value: i32) -> Result<String, Error<dyn IThrowable>> {
//!     if value == 1 {
//!         throw_string!("Error! Value is one!");
//!     }
//!     Ok(format!("All ok! Value: {}", value))
//! }
//!
//! fn foo(value: i32) -> Result<String, Error<dyn IThrowable>> {
//!     if value == 0 {
//!         throw_string!("Error! Value is zero!");
//!     }
//!     Ok(catch!(bar(value)))
//! }
//!
//! fn run() -> Result<(), Error<dyn IThrowable>> {
//!     println!("{}", catch!(foo(21)));   // Display: only the ok value
//!     println!("{}", catch!(foo(1)));    // Display: only error message
//!     Ok(())
//! }
//!
//! # fn main() {
//! #     if let Err(e) = run() {
//! #         // Show only the error message
//! #         println!("Error: {}", e);
//! #         // Show full chain with traces
//! #         println!("Full trace: {:?}", e);
//! #     }
//! # }
//! ```
//!
//! ## Accessing error data
//! Use the `throwable()` method on `Error` to get a reference to the inner payload
//! and call custom methods. The following example uses a concrete error type `MyError`
//! so that its methods are accessible:
//! ```
//! # use dyn_trace_err::{Error, IThrowable, trace::Trace};
//! # use std::fmt;
//! # #[derive(Debug)]
//! # struct MyError { code: u32 }
//! # impl IThrowable for MyError { fn cause(&self) -> &Option<Box<Error<dyn IThrowable>>> { &None } }
//! # impl fmt::Display for MyError { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "err") } }
//! # impl MyError { fn code(&self) -> u32 { self.code } }
//! # #[cfg(not(feature = "no-trace"))] {
//! let err = Error::new(Box::new(MyError { code: 404 }), Trace::new("test".to_string(), None));
//! assert_eq!(err.throwable().code(), 404);
//! # }
//! # #[cfg(feature = "no-trace")] {
//! # let err = Error::new(Box::new(MyError { code: 404 }));
//! # assert_eq!(err.throwable().code(), 404);
//! # }
//! ```
//!
//! ## Built‑in error types
//! - [`StringException`](r#impl::StringException) – stores a simple string message.
//! - [`FormattableException`](r#impl::FormattableException) – wraps any type that implements
//!   both `Display` and `Debug`, allowing different representations.
//! - [`FormattableStringException`](r#impl::FormattableStringException) – stores separate
//!   `Display` and `Debug` string messages, created via the
//!   [`throw_formattable_string!`] macro.

#![no_std]

// Ensure exactly one tracing feature is selected.
#[cfg(any(
    all(feature = "all-trace", feature = "my-trace"),
    all(feature = "all-trace", feature = "no-trace"),
    all(feature = "my-trace", feature = "no-trace"),
))]
compile_error!("Only one of the features 'all-trace', 'my-trace', 'no-trace' can be enabled at a time.");

#[cfg(not(any(
    feature = "all-trace",
    feature = "my-trace",
    feature = "no-trace",
)))]
compile_error!("One of the features 'all-trace', 'my-trace', 'no-trace' must be enabled.");

pub mod r#impl;
#[cfg(not(feature = "no-trace"))]
pub mod trace;

extern crate alloc;

pub use alloc::boxed::Box;
use core::fmt::{Debug, Display, Formatter};

/// The main error type.
///
/// Contains the payload (`throw`) that implements [`IThrowable`], and (unless `no-trace` is enabled)
/// a stack trace ([`Trace`](trace::Trace)).
///
/// # Formatting
/// - `Display` → prints only the error message (from `throw`).
/// - `Debug`   → prints the full error chain with all causes and stack traces.
///
/// # Type parameter
/// `T` must implement `IThrowable + ?Sized`. Most commonly you will use `dyn IThrowable`
/// for dynamic dispatch, but you can also use concrete types if you don't need polymorphism.
///
/// # Examples
/// ```
/// # use dyn_trace_err::{Error, IThrowable, throw_string};
/// # fn example() -> Result<(), Error<dyn IThrowable>> {
/// throw_string!("An error");
/// # Ok(())
/// # }
/// ```
pub struct Error<T: IThrowable + ?Sized> {
    /// The error object implementing [`IThrowable`].
    throw: Box<T>,
    /// The stack trace (absent when `no-trace` is enabled).
    #[cfg(not(feature = "no-trace"))]
    trace: trace::Trace,
}

/// A type alias for [`Error`] with dynamic dispatch over [`IThrowable`].
///
/// `AnyError` is the most commonly used error type in this crate. It allows you to
/// hold any concrete error type that implements [`IThrowable`] without specifying
/// the exact type, making it convenient for functions that want to return a generic
/// error or propagate errors from different sources.
///
/// This type is especially useful with the provided macros (`throw!`, `catch!`,
/// `throw_string!`, `throw_formattable!`), which by default return `Result<T, AnyError>`.
///
/// # Example
/// ```
/// use dyn_trace_err::{AnyError, throw_string, catch, IThrowable};
///
/// fn fallible(flag: bool) -> Result<(), AnyError> {
///     if flag {
///         throw_string!("Something went wrong");
///     }
///     Ok(())
/// }
///
/// fn caller() -> Result<(), AnyError> {
///     catch!(fallible(true));
///     Ok(())
/// }
///
/// # fn main() {
/// #     if let Err(e) = caller() {
/// #         eprintln!("Error: {}", e);
/// #     }
/// # }
/// ```
///
/// # Accessing the inner payload
/// You can use the [`throwable()`](Error::throwable) method to get a reference to
/// the inner error object and call its custom methods (if you downcast or know
/// the concrete type).
///
/// # Tracing
/// The behavior of `AnyError` regarding stack traces depends on the enabled
/// feature (`all-trace`, `my-trace`, or `no-trace`). Refer to the crate-level
/// documentation for details.
pub type AnyError = Error<dyn IThrowable>;

/// Helper wrapper for formatting an error chain with indentation.
struct ErrorDisplayWrapper<'a> {
    error: &'a Error<dyn IThrowable>,
    inner: usize,
}

/// Trait that all error types used in [`Error`] must implement.
///
/// It enables chaining of causes via the `cause()` method.
///
/// # Example
/// ```
/// # use dyn_trace_err::{IThrowable, Error};
/// # use std::fmt;
/// # #[derive(Debug)]
/// # struct MyError {
/// #     msg: String,
/// #     cause: Option<Box<Error<dyn IThrowable>>>,
/// # }
/// # impl IThrowable for MyError {
/// #     fn cause(&self) -> &Option<Box<Error<dyn IThrowable>>> {
/// #         &self.cause
/// #     }
/// # }
/// # impl fmt::Display for MyError {
/// #     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
/// #         write!(f, "{}", self.msg)
/// #     }
/// # }
/// ```
pub trait IThrowable: Debug + Display {
    /// Returns a reference to the nested error (cause), if any.
    fn cause(&self) -> &Option<Box<Error<dyn IThrowable>>>;
}

impl<T: IThrowable + ?Sized> Error<T> {
    /// Creates a new error.
    ///
    /// # Parameters
    /// - `throw` – the error payload.
    /// - `trace` – the trace (available only when `!no-trace`).
    #[cfg(not(feature = "no-trace"))]
    #[inline(always)]
    pub fn new(throw: Box<T>, trace: trace::Trace) -> Self {
        Self { throw, trace }
    }

    /// Creates a new error (trace‑less version).
    #[cfg(feature = "no-trace")]
    pub fn new(throw: Box<T>) -> Self {
        Self { throw }
    }

    /// Adds a new trace point to an existing error.
    ///
    /// Takes a function that receives the previous `Trace` and returns a new one.
    /// Available only when `!no-trace`.
    #[cfg(not(feature = "no-trace"))]
    #[inline(always)]
    pub fn trace_point(self, trace: fn(prev: trace::Trace) -> trace::Trace) -> Self {
        Self {
            throw: self.throw,
            trace: trace(self.trace),
        }
    }

    /// Returns a reference to the inner error payload.
    ///
    /// This allows you to access custom data or methods defined on your error type.
    ///
    /// # Example
    /// ```
    /// # use dyn_trace_err::{Error, IThrowable, trace::Trace};
    /// # use std::fmt;
    /// # #[derive(Debug)]
    /// # struct MyError { code: u32 }
    /// # impl IThrowable for MyError { fn cause(&self) -> &Option<Box<Error<dyn IThrowable>>> { &None } }
    /// # impl fmt::Display for MyError { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "err") } }
    /// # impl MyError { fn code(&self) -> u32 { self.code } }
    /// # #[cfg(not(feature = "no-trace"))] {
    /// let err = Error::new(Box::new(MyError { code: 404 }), Trace::new("test".to_string(), None));
    /// assert_eq!(err.throwable().code(), 404);
    /// # }
    /// # #[cfg(feature = "no-trace")] {
    /// # let err = Error::new(Box::new(MyError { code: 404 }));
    /// # assert_eq!(err.throwable().code(), 404);
    /// # }
    /// ```
    #[inline(always)]
    pub fn throwable(&self) -> &T {
        &self.throw
    }

    /// Internal formatting helper that respects the nesting level.
    fn display(&self, fmt: &mut Formatter<'_>, inner: usize) -> core::fmt::Result {
        write!(fmt, "[{}] {:?}", inner, self.throw)?;
        #[cfg(not(feature = "no-trace"))]
        write!(fmt, "\n{}", self.trace)?;
        if let Some(cause) = self.throw.cause() {
            write!(fmt, "\n{:?}", ErrorDisplayWrapper {
                error: cause,
                inner: inner + 1,
            })?;
        }
        Ok(())
    }
}

impl Display for Error<dyn IThrowable> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.throw, f)
    }
}

impl Debug for Error<dyn IThrowable> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.display(f, 0)
    }
}

impl Debug for ErrorDisplayWrapper<'_> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.error.display(f, self.inner)
    }
}

/// Creates an error and immediately returns `Result` with `Err`.
///
/// ## Available forms (depend on the feature)
///
/// ### `all-trace`
/// - `throw!($expr)` – automatically adds a trace with `file!()` and `line!()`.
/// - `throw!($expr, $trace)` – adds the given `Trace`.
///
/// ### `my-trace`
/// - `throw!($expr, $trace)` – requires an explicit `Trace`.
///
/// ### `no-trace`
/// - `throw!($expr)` – creates an error without a trace.
///
/// # Example
/// ```
/// # use dyn_trace_err::{throw, Error, IThrowable};
/// # use core::fmt::{self, Display};
/// # #[derive(Debug)]
/// # struct MyError;
/// # impl Display for MyError { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "MyError") } }
/// # impl IThrowable for MyError { fn cause(&self) -> &Option<Box<Error<dyn IThrowable>>> { &None } }
/// # fn example() -> Result<(), Error<dyn IThrowable>> {
/// throw!(Box::new(MyError));
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "all-trace")]
#[macro_export]
macro_rules! throw {
    ($expr:expr) => {
        return Err($crate::Error::new($expr, $crate::trace::Trace::new(format!("{}:{}", file!(), line!()), None)));
    };
    ($expr:expr, $trace:expr) => {
        return Err($crate::Error::new($expr, $trace));
    };
}

#[cfg(feature = "my-trace")]
#[macro_export]
macro_rules! throw {
    ($expr:expr, $trace:expr) => {
        return Err($crate::Error::new($expr, $trace));
    };
}

#[cfg(feature = "no-trace")]
#[macro_export]
macro_rules! throw {
    ($expr:expr) => {
        return Err($crate::Error::new($expr));
    };
}

/// Evaluates an expression that returns a `Result` and, on error, adds a new trace point.
///
/// ## Available forms (depend on the feature)
///
/// ### `all-trace`
/// - `catch!($expr)` – adds a point with `file!()` and `line!()`.
/// - `catch!($expr, $trace)` – adds the given `Trace`.
///
/// ### `my-trace`
/// - `catch!($expr, $trace)` – requires an explicit `Trace`.
///
/// ### `no-trace`
/// - `catch!($expr)` – simply propagates the error unchanged.
///
/// # Example
/// ```
/// # use dyn_trace_err::{catch, throw_string, Error, IThrowable};
/// # fn fallible() -> Result<(), Error<dyn IThrowable>> { throw_string!("fail") }
/// # fn example() -> Result<(), Error<dyn IThrowable>> {
/// catch!(fallible());
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "all-trace")]
#[macro_export]
macro_rules! catch {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => return Err(e.trace_point(|prev| $crate::trace::Trace::new(format!("{}:{}", file!(), line!()), Some(prev))))
        }
    };
    ($expr:expr, $trace:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => return Err(e.trace_point($trace))
        }
    };
}

#[cfg(feature = "my-trace")]
#[macro_export]
macro_rules! catch {
    ($expr:expr, $trace:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => return Err(e.trace_point($trace))
        }
    };
}

#[cfg(feature = "no-trace")]
#[macro_export]
macro_rules! catch {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => return Err(e)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r#impl::StringException;
    use alloc::format;
    use alloc::string::ToString;
    use alloc::vec::Vec;

    /// Проверяем, что макрос throw! возвращает ошибку с правильным сообщением и трейсом
    #[test]
    fn test_throw_macro_creates_error() {
        fn failing_function() -> Result<(), Error<dyn IThrowable>> {
            let msg = "Something went wrong".to_string();
            throw!(StringException::new(msg, None));
        }

        let result = failing_function();
        assert!(result.is_err());

        let err = result.unwrap_err();
        let actual_message = format!("{}", err.throw);
        assert_eq!(actual_message, "Something went wrong");

        #[cfg(not(feature = "no-trace"))]
        {
            let trace = &err.trace;
            let point = &trace.point;
            assert!(!point.is_empty());
            assert!(point.contains(':'));
            let parts: Vec<&str> = point.split(':').collect();
            assert_eq!(parts.len(), 2);
            let line: usize = parts[1].parse().expect("Номер строки должен быть числом");
            assert!(line > 0);
            assert!(parts[0].contains("lib.rs") || parts[0].contains("tests"));
            assert!(trace.prev.is_none());
        }
    }

    /// Проверяем, что макрос throw! работает с причиной (cause)
    #[test]
    fn test_throw_with_cause() {
        fn inner() -> Result<(), Error<dyn IThrowable>> {
            throw!(StringException::new("Inner error".to_string(), None));
        }

        fn outer() -> Result<(), Error<dyn IThrowable>> {
            if let Err(e) = inner() {
                throw!(StringException::new("Outer error".to_string(), Some(e)));
            }
            Ok(())
        }

        let result = outer();
        assert!(result.is_err());
        let err = result.unwrap_err();

        let cause_opt = err.throw.cause();
        assert!(cause_opt.is_some());
        let cause_err = cause_opt.as_ref().unwrap();
        let cause_msg = format!("{}", cause_err.throw);
        assert_eq!(cause_msg, "Inner error");

        #[cfg(not(feature = "no-trace"))]
        {
            assert!(!err.trace.point.is_empty());
            assert!(!cause_err.trace.point.is_empty());
        }
    }

    /// Проверяет throw_string! без причины
    #[test]
    fn test_throw_string_macro_without_cause() {
        fn failing() -> Result<(), Error<dyn IThrowable>> {
            throw_string!("Simple error");
        }

        let result = failing();
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{}", err.throw);
        assert_eq!(msg, "Simple error");
    }

    /// Проверяет throw_string! с причиной
    #[test]
    fn test_throw_string_macro_with_cause() {
        fn inner() -> Result<(), Error<dyn IThrowable>> {
            throw_string!("Inner error");
        }

        fn outer() -> Result<(), Error<dyn IThrowable>> {
            if let Err(e) = inner() {
                throw_string!("Outer error", Some(e));
            }
            Ok(())
        }

        let result = outer();
        assert!(result.is_err());
        let err = result.unwrap_err();

        let cause_opt = err.throw.cause();
        assert!(cause_opt.is_some());
        let cause_err = cause_opt.as_ref().unwrap();
        let msg = format!("{}", cause_err.throw);
        assert_eq!(msg, "Inner error");
    }

    /// Проверяет, что catch! добавляет точку трассировки
    #[test]
    fn test_catch_macro_adds_trace_point() {
        fn inner() -> Result<(), Error<dyn IThrowable>> {
            throw!(StringException::new("Inner error".to_string(), None));
        }

        fn outer() -> Result<(), Error<dyn IThrowable>> {
            catch!(inner());
            Ok(())
        }

        let result = outer();
        assert!(result.is_err());

        let err = result.unwrap_err();

        #[cfg(not(feature = "no-trace"))]
        {
            let trace = &err.trace;
            assert!(trace.prev.is_some());
            assert!(trace.point.contains(':'));
            if let Some(prev_trace) = &trace.prev {
                assert!(prev_trace.point.contains(':'));
            }
            let mut count = 0;
            let mut current = Some(trace);
            while let Some(t) = current {
                count += 1;
                current = t.prev.as_deref();
            }
            assert!(count >= 2);
        }

        #[cfg(feature = "no-trace")]
        {
            let msg = format!("{}", err.throw);
            assert_eq!(msg, "Inner error");
        }
    }

    /// Проверяет правильность форматирования цепочки ошибок в Debug (полная цепочка)
    /// и Display (только сообщение)
    #[test]
    fn test_display_formatting_with_cause() {
        fn inner() -> Result<(), Error<dyn IThrowable>> {
            throw_string!("Inner");
        }
        fn outer() -> Result<(), Error<dyn IThrowable>> {
            if let Err(e) = inner() {
                throw_string!("Outer", Some(e));
            }
            Ok(())
        }

        let err = outer().unwrap_err();

        // Display должен выводить только сообщение внешней ошибки
        let display_output = format!("{}", err);
        assert_eq!(display_output, "Outer");

        // Debug должен выводить полную цепочку с причинами и трассами
        let debug_output = format!("{:?}", err);
        assert!(debug_output.contains("[0] Outer"));
        assert!(debug_output.contains("[1] Inner"));

        #[cfg(not(feature = "no-trace"))]
        {
            let count = debug_output.matches("| [0]").count();
            assert!(count >= 2, "Должно быть как минимум две строки с | [0] (для двух ошибок)");
            let pos0 = debug_output.find("[0] Outer").unwrap();
            let pos1 = debug_output.find("[1] Inner").unwrap();
            assert!(pos0 < pos1, "Сообщение внешней ошибки должно идти раньше внутренней");
        }
    }

    /// Проверяет порядок точек трассировки (только с трассировкой)
    #[test]
    #[cfg(not(feature = "no-trace"))]
    fn test_trace_order() {
        fn level3() -> Result<(), Error<dyn IThrowable>> {
            throw_string!("level3");
        }
        fn level2() -> Result<(), Error<dyn IThrowable>> {
            catch!(level3());
            Ok(())
        }
        fn level1() -> Result<(), Error<dyn IThrowable>> {
            catch!(level2());
            Ok(())
        }

        let err = level1().unwrap_err();
        let trace = &err.trace;
        let mut count = 0;
        let mut current = Some(trace);
        while let Some(t) = current {
            count += 1;
            assert!(t.point.contains(':'), "Точка должна быть в формате file:line");
            current = t.prev.as_deref();
        }
        assert_eq!(count, 3, "Должно быть ровно три точки трассировки");
    }

    /// Проверяет явный вызов метода trace_point
    #[test]
    #[cfg(not(feature = "no-trace"))]
    fn test_trace_point_method() {
        use crate::trace::Trace;

        let err = Error::new(
            StringException::new("base".to_string(), None),
            Trace::new("root".to_string(), None),
        );
        let new_err = err.trace_point(|prev| Trace::new("new_point".to_string(), Some(prev)));

        assert_eq!(new_err.trace.point, "new_point");
        let prev = new_err.trace.prev.as_ref().unwrap();
        assert_eq!(prev.point, "root");
        assert!(prev.prev.is_none());
    }

    /// Проверяет catch! с явным трейсом (доступно в all-trace и my-trace)
    #[test]
    #[cfg(any(feature = "all-trace", feature = "my-trace"))]
    fn test_catch_with_explicit_trace() {
        use crate::trace::Trace;

        fn inner() -> Result<(), Error<dyn IThrowable>> {
            throw!(StringException::new("inner".to_string(), None));
        }

        fn outer() -> Result<(), Error<dyn IThrowable>> {
            catch!(inner(), |prev| Trace::new("explicit".to_string(), Some(prev)));
            Ok(())
        }

        let err = outer().unwrap_err();
        #[cfg(not(feature = "no-trace"))]
        {
            assert_eq!(err.trace.point, "explicit");
            let prev = err.trace.prev.as_ref().unwrap();
            assert!(prev.point.contains(':') || prev.point == "inner");
        }
    }

    /// Проверяет throw_string! с явным трейсом (доступно в all-trace/my-trace)
    #[test]
    #[cfg(any(feature = "all-trace", feature = "my-trace"))]
    fn test_throw_string_with_explicit_trace() {
        use crate::trace::Trace;

        fn fail() -> Result<(), Error<dyn IThrowable>> {
            throw_string!("msg", None, Trace::new("custom_trace".to_string(), None));
        }

        let err = fail().unwrap_err();
        #[cfg(not(feature = "no-trace"))]
        {
            assert_eq!(err.trace.point, "custom_trace");
            assert!(err.trace.prev.is_none());
        }
    }

    /// Проверяет, что при no-trace ошибка не содержит trace
    #[test]
    #[cfg(feature = "no-trace")]
    fn test_no_trace_absence() {
        let err = Error::new(StringException::new("test".to_string(), None));
        let output = format!("{}", err);
        assert!(!output.contains("| ["));
        assert!(output.contains("[0] test"));
    }

    /// Проверка метода throwable() для доступа к данным ошибки
    #[test]
    fn test_throwable_method() {
        use crate::r#impl::StringException;

        fn fail() -> Result<(), Error<dyn IThrowable>> {
            throw!(StringException::new("test".to_string(), None));
        }

        let err = fail().unwrap_err();
        let throwable = err.throwable();
        let msg = format!("{}", throwable);
        assert_eq!(msg, "test");
    }

    /// Проверка пользовательского типа, реализующего IThrowable
    #[test]
    #[cfg(not(feature = "no-trace"))]
    fn test_custom_throwable_type() {
        use crate::trace::Trace;
        use core::fmt;

        struct MyErr {
            code: u32,
            cause: Option<Box<Error<dyn IThrowable>>>,
        }
        impl IThrowable for MyErr {
            fn cause(&self) -> &Option<Box<Error<dyn IThrowable>>> {
                &self.cause
            }
        }
        impl Display for MyErr {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                write!(f, "Code: {}", self.code)
            }
        }
        impl Debug for MyErr {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                write!(f, "Code: {}", self.code)
            }
        }

        let err = MyErr { code: 42, cause: None };
        let error = Error::new(Box::new(err), Trace::new("test".to_string(), None));
        let throwable = error.throwable();
        assert_eq!(throwable.code, 42);
    }

    /// Тест для FormattableException и макроса throw_formattable!
    #[test]
    fn test_formattable_exception() {
        use crate::r#impl::Formattable;
        use core::fmt;

        struct CustomError;
        impl Display for CustomError {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                write!(f, "Display message")
            }
        }
        impl Debug for CustomError {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                write!(f, "Debug message")
            }
        }
        impl Formattable for CustomError {}

        fn fail() -> Result<(), Error<dyn IThrowable>> {
            throw_formattable!(CustomError);
        }

        let err = fail().unwrap_err();
        // Display должен использовать Display impl
        assert_eq!(format!("{}", err), "Display message");
        // Debug должен использовать Debug impl
        assert!(format!("{:?}", err).contains("Debug message"));
    }

    /// Проверяет FormattableStringException и макрос throw_formattable_string!
    #[test]
    fn test_formattable_string_exception() {
        use crate::r#impl::FormattableStringException;

        // Создание через new
        let err = FormattableStringException::new(
            "display message".to_string(),
            "debug message".to_string(),
            None,
        );
        assert_eq!(format!("{}", err), "display message");
        assert_eq!(format!("{:?}", err), "debug message");

        // Создание через макрос
        fn fail() -> Result<(), Error<dyn IThrowable>> {
            throw_formattable_string!("User error", "Debug: detailed info");
        }
        let err = fail().unwrap_err();
        assert_eq!(format!("{}", err), "User error");
        assert!(format!("{:?}", err).contains("Debug: detailed info"));
    }

    /// Проверяет цепочку причин с FormattableStringException
    #[test]
    fn test_formattable_string_with_cause() {
        fn inner() -> Result<(), Error<dyn IThrowable>> {
            throw_formattable_string!("inner display", "inner debug");
        }
        fn outer() -> Result<(), Error<dyn IThrowable>> {
            if let Err(e) = inner() {
                throw_formattable_string!("outer display", "outer debug", Some(e));
            }
            Ok(())
        }

        let err = outer().unwrap_err();
        // Display должен показывать только внешнее сообщение
        assert_eq!(format!("{}", err), "outer display");

        // Debug должен содержать обе ошибки
        let debug = format!("{:?}", err);
        assert!(debug.contains("outer debug"));
        assert!(debug.contains("inner debug"));
        // Проверяем, что причина присутствует
        assert!(err.throwable().cause().is_some());
        let cause = err.throwable().cause().as_ref().unwrap();
        assert_eq!(format!("{}", cause), "inner display");
    }

    /// Проверяет, что throw_formattable_string! работает с явным трейсом (если не no-trace)
    #[test]
    #[cfg(any(feature = "all-trace", feature = "my-trace"))]
    fn test_formattable_string_with_explicit_trace() {
        use crate::trace::Trace;

        fn fail() -> Result<(), Error<dyn IThrowable>> {
            throw_formattable_string!(
                "display",
                "debug",
                None,
                Trace::new("custom_trace".to_string(), None)
            );
        }

        let err = fail().unwrap_err();
        #[cfg(not(feature = "no-trace"))]
        {
            assert_eq!(err.trace.point, "custom_trace");
        }
    }
}