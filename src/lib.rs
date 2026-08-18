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
//! - Convenient macros: `throw!`, `catch!`, and `throw_string!`.
//!
//! ## Example
//! ```
//! use dyn_trace_err::{throw_string, catch, Error};
//!
//! fn bar(value: i32) -> Result<String, Error> {
//!     if value == 1 {
//!         throw_string!("Error! Value is one!");
//!     }
//!     Ok(format!("All ok! Value: {}", value))
//! }
//!
//! fn foo(value: i32) -> Result<String, Error> {
//!     if value == 0 {
//!         throw_string!("Error! Value is zero!");
//!     }
//!     Ok(catch!(bar(value)))
//! }
//!
//! # fn main() -> Result<(), Error> {
//! println!("{}", catch!(foo(21))?);
//! println!("{}", catch!(foo(1))?); // this will error
//! # Ok(())
//! # }
//! ```

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

use alloc::boxed::Box;
use core::fmt::{Display, Formatter};

/// The main error type.
///
/// Contains the payload (`throw`) that implements [`IThrowable`], and (unless `no-trace` is enabled)
/// a stack trace ([`Trace`](trace::Trace)).
///
/// Implements [`Display`] to pretty‑print the entire error chain with traces.
pub struct Error {
    /// The error object implementing [`IThrowable`].
    pub throw: Box<dyn IThrowable>,
    /// The stack trace (absent when `no-trace` is enabled).
    #[cfg(not(feature = "no-trace"))]
    pub trace: trace::Trace,
}

/// Helper wrapper for formatting an error chain with indentation.
struct ErrorDisplayWrapper<'a> {
    error: &'a Error,
    inner: usize,
}

/// Trait that all error types used in [`Error`] must implement.
///
/// It enables chaining of causes via the `cause()` method.
pub trait IThrowable: Display {
    /// Returns a reference to the nested error (cause), if any.
    fn cause(&self) -> &Option<Box<Error>>;
}

impl Error {
    /// Creates a new error.
    ///
    /// # Parameters
    /// - `throw` – the error payload.
    /// - `trace` – the trace (available only when `!no-trace`).
    #[cfg(not(feature = "no-trace"))]
    #[inline(always)]
    pub fn new(throw: Box<dyn IThrowable>, trace: trace::Trace) -> Self {
        Self { throw, trace }
    }

    /// Creates a new error (trace‑less version).
    #[cfg(feature = "no-trace")]
    pub fn new(throw: Box<dyn IThrowable>) -> Self {
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

    /// Internal formatting helper that respects the nesting level.
    fn display(&self, fmt: &mut Formatter<'_>, inner: usize) -> core::fmt::Result {
        write!(fmt, "[{}] {}", inner, self.throw)?;
        #[cfg(not(feature = "no-trace"))]
        write!(fmt, "\n{}", self.trace)?;
        if let Some(cause) = self.throw.cause() {
            write!(fmt, "\n{}", ErrorDisplayWrapper {
                error: cause,
                inner: inner + 1,
            })?;
        }
        Ok(())
    }
}

impl Display for Error {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.display(f, 0)
    }
}

impl Display for ErrorDisplayWrapper<'_> {
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
/// # use core::fmt;
/// # struct MyError;
/// # impl Display for MyError { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "MyError") } }
/// # impl IThrowable for MyError { fn cause(&self) -> &Option<Box<Error>> { &None } }
/// # fn example() -> Result<(), Error> {
/// throw!(MyError);
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
/// # use dyn_trace_err::{catch, throw_string, Error};
/// # fn fallible() -> Result<(), Error> { Err(throw_string!("fail")) }
/// # fn example() -> Result<(), Error> {
/// catch!(fallible())?;
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

    /// Проверяем, что макрос возвращает ошибку с правильным сообщением и трейсом
    #[test]
    fn test_throw_macro_creates_error() {
        fn failing_function() -> Result<(), Error> {
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

    /// Проверяем, что макрос работает с причиной (cause)
    #[test]
    fn test_throw_with_cause() {
        fn inner() -> Result<(), Error> {
            throw!(StringException::new("Inner error".to_string(), None));
        }

        fn outer() -> Result<(), Error> {
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

    #[test]
    fn test_throw_string_macro_without_cause() {
        fn failing() -> Result<(), Error> {
            throw_string!("Simple error");
        }

        let result = failing();
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{}", err.throw);
        assert_eq!(msg, "Simple error");
    }

    #[test]
    fn test_throw_string_macro_with_cause() {
        fn inner() -> Result<(), Error> {
            throw_string!("Inner error");
        }

        fn outer() -> Result<(), Error> {
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

    #[test]
    fn test_catch_macro_adds_trace_point() {
        fn inner() -> Result<(), Error> {
            throw!(StringException::new("Inner error".to_string(), None));
        }

        fn outer() -> Result<(), Error> {
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

    /// Проверяет правильность форматирования цепочки ошибок (Display)
    #[test]
    fn test_display_formatting_with_cause() {
        fn inner() -> Result<(), Error> {
            throw_string!("Inner");
        }
        fn outer() -> Result<(), Error> {
            if let Err(e) = inner() {
                throw_string!("Outer", Some(e));
            }
            Ok(())
        }

        let err = outer().unwrap_err();
        let output = format!("{}", err);
        // Должны присутствовать оба сообщения
        assert!(output.contains("[0] Outer"));
        assert!(output.contains("[1] Inner"));
        // Если трассировка включена, должны быть строки с "| [0]" для каждой ошибки
        #[cfg(not(feature = "no-trace"))]
        {
            // Проверяем, что после [0] Outer есть | [0] (трасса внешней ошибки)
            // и после [1] Inner есть | [0] (трасса внутренней ошибки)
            // Можно просто проверить наличие двух строк с "| [0]"
            let count = output.matches("| [0]").count();
            assert!(count >= 2, "Должно быть как минимум две строки с | [0] (для двух ошибок)");
            // Убедимся, что порядок правильный: сначала внешняя, потом внутренняя
            let pos0 = output.find("[0] Outer").unwrap();
            let pos1 = output.find("[1] Inner").unwrap();
            assert!(pos0 < pos1, "Сообщение внешней ошибки должно идти раньше внутренней");
        }
        // При no-trace трасс нет, но цепочка причин всё равно отображается
    }

    /// Проверяет, что порядок точек трассировки соответствует порядку вызовов
    #[test]
    #[cfg(not(feature = "no-trace"))] // тест имеет смысл только с трассировкой
    fn test_trace_order() {
        fn level3() -> Result<(), Error> {
            throw_string!("level3");
        }
        fn level2() -> Result<(), Error> {
            catch!(level3());
            Ok(())
        }
        fn level1() -> Result<(), Error> {
            catch!(level2());
            Ok(())
        }

        let err = level1().unwrap_err();
        let trace = &err.trace;
        // Проверяем, что есть три точки: level1, level2, level3
        // При all-trace каждая точка имеет вид "file:line"
        let mut count = 0;
        let mut current = Some(trace);
        while let Some(t) = current {
            count += 1;
            // Проверяем, что точка содержит двоеточие (формат file:line)
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

        fn inner() -> Result<(), Error> {
            throw!(StringException::new("inner".to_string(), None));
        }

        fn outer() -> Result<(), Error> {
            catch!(inner(), |prev| Trace::new("explicit".to_string(), Some(prev)));
            Ok(())
        }

        let err = outer().unwrap_err();
        // Проверяем, что добавлена явная точка
        #[cfg(not(feature = "no-trace"))]
        {
            assert_eq!(err.trace.point, "explicit");
            let prev = err.trace.prev.as_ref().unwrap();
            // предыдущая точка должна быть от throw! (автоматическая или тоже заданная)
            // Мы не знаем точное содержимое, но она должна существовать
            assert!(prev.point.contains(':') || prev.point == "inner");
        }
    }

    /// Проверяет throw_string! с явным трейсом (доступно в all-trace/my-trace)
    #[test]
    #[cfg(any(feature = "all-trace", feature = "my-trace"))]
    fn test_throw_string_with_explicit_trace() {
        use crate::trace::Trace;

        fn fail() -> Result<(), Error> {
            throw_string!("msg", None, Trace::new("custom_trace".to_string(), None));
        }

        let err = fail().unwrap_err();
        #[cfg(not(feature = "no-trace"))]
        {
            assert_eq!(err.trace.point, "custom_trace");
            assert!(err.trace.prev.is_none());
        }
    }

    /// Проверяет, что при no-trace ошибка не содержит trace (и методы не паникуют)
    #[test]
    #[cfg(feature = "no-trace")]
    fn test_no_trace_absence() {
        let err = Error::new(StringException::new("test".to_string(), None));
        // Проверяем, что поле trace отсутствует на уровне типов (это проверяется компиляцией)
        // Также проверим, что Display не содержит строк трассировки
        let output = format!("{}", err);
        assert!(!output.contains("| ["));
        assert!(output.contains("[0] test"));
    }
}