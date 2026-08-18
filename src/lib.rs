#![no_std]

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
use alloc::string::String;
use core::fmt::{Display, Formatter};

pub struct Error {
    pub throw: Box<dyn IThrowable>,
    #[cfg(not(feature = "no-trace"))]
    pub trace: trace::Trace
}

struct ErrorDisplayWrapper<'a> {
    error: &'a Error,
    inner: usize
}

pub trait IThrowable : Display {
    fn cause(&self) -> &Option<Box<Error>>;
}

impl Error {
    #[cfg(not(feature = "no-trace"))]
    #[inline(always)]
    pub fn new(throw: Box<dyn IThrowable>, trace: trace::Trace) -> Self {
        Self { throw, trace }
    }

    #[cfg(feature = "no-trace")]
    pub fn new(throw: Box<dyn IThrowable>) -> Self {
        Self { throw }
    }

    #[cfg(not(feature = "no-trace"))]
    #[inline(always)]
    pub fn trace_point(self, point: String) -> Self {
        Self { throw: self.throw, trace: trace::Trace::new(point, Some(self.trace)) }
    }

    fn display(&self, fmt: &mut Formatter<'_>, inner: usize) -> core::fmt::Result {
        write!(fmt, "[{}] {}", inner, self.throw)?;
        #[cfg(not(feature = "no-trace"))]
        write!(fmt, "\n{}", self.trace)?;
        if let Some(cause) = self.throw.cause() {
            write!(fmt, "\n{}", ErrorDisplayWrapper { error: cause, inner: inner + 1 })?;
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

#[cfg(not(feature = "no-trace"))]
#[macro_export]
macro_rules! throw {
    ($expr:expr) => {
        return Err($crate::Error::new($expr, $crate::trace::Trace::new(format!("{}:{}", file!(), line!()), None)));
    };
}

#[cfg(feature = "no-trace")]
#[macro_export]
macro_rules! throw {
    ($expr:expr) => {
        return Err($crate::Error::new($expr));
    };
}

#[macro_export]
macro_rules! catch {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => return Err(e.trace_point(format!("{}:{}", file!(), line!())))
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
    use alloc::string::ToString;
    use alloc::format;
    use alloc::vec::Vec;

    /// Проверяем, что макрос возвращает ошибку с правильным сообщением и трейсом
    #[test]
    fn test_throw_macro_creates_error() {
        // Функция, в которой используется throw!
        fn failing_function() -> Result<(), Error> {
            let msg = "Something went wrong".to_string();
            // Создаём исключение без причины
            throw!(StringException::new(msg, None));
            // Эта строка никогда не выполнится
            #[allow(unreachable_code)]
            Ok(())
        }

        let result = failing_function();
        assert!(result.is_err(), "Должна быть ошибка");

        let err = result.unwrap_err();

        // Проверяем сообщение ошибки через Display поля throw
        let throwable = &err.throw;
        let actual_message = format!("{}", throwable);
        assert_eq!(actual_message, "Something went wrong");

        // Проверяем наличие трейса, если он не отключён
        #[cfg(not(feature = "no-trace"))]
        {
            let trace = &err.trace;
            // point должен быть строкой вида "file:line"
            let point = &trace.point;
            assert!(!point.is_empty(), "point не должен быть пустым");
            assert!(point.contains(':'), "point должен содержать двоеточие");

            // Разбираем на части
            let parts: Vec<&str> = point.split(':').collect();
            assert_eq!(parts.len(), 2, "формат должен быть file:line");

            // Номер строки должен быть положительным числом
            let line: usize = parts[1]
                .parse()
                .expect("Номер строки должен быть числом");
            assert!(line > 0, "номер строки должен быть > 0");

            // Можно дополнительно проверить, что файл соответствует текущему
            // (например, содержит "lib.rs" или имя тестового модуля)
            assert!(
                parts[0].contains("lib.rs") || parts[0].contains("tests"),
                "имя файла должно указывать на исходный код"
            );

            // Проверяем, что prev == None (это корневой трейс)
            assert!(trace.prev.is_none(), "prev должен быть None для корневого трейса");
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

        // Проверяем, что причина есть
        let cause_opt = err.throw.cause();
        assert!(cause_opt.is_some(), "должна быть причина");

        let cause_err = cause_opt.as_ref().unwrap();
        let cause_msg = format!("{}", cause_err.throw);
        assert_eq!(cause_msg, "Inner error");

        // Трейсы будут у каждой ошибки, если no-trace не включена
        #[cfg(not(feature = "no-trace"))]
        {
            // У внешней ошибки тоже должен быть свой трейс
            assert!(!err.trace.point.is_empty());
            // У внутренней ошибки тоже есть трейс (внутри cause_err)
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
                throw_string!("Outer error", e);
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
        // Внутренняя функция, которая всегда возвращает ошибку через throw!
        fn inner() -> Result<(), Error> {
            throw!(StringException::new("Inner error".to_string(), None));
        }

        // Внешняя функция, использующая catch! для вызова inner
        fn outer() -> Result<(), Error> {
            // Добавляем трейс в текущей точке
            catch!(inner());
            Ok(())
        }

        let result = outer();
        assert!(result.is_err());

        let err = result.unwrap_err();

        #[cfg(not(feature = "no-trace"))]
        {
            let trace = &err.trace;

            // Проверяем, что добавлена новая точка (prev не None)
            assert!(
                trace.prev.is_some(),
                "try! должен добавить новую точку поверх существующего трейса"
            );

            // Проверяем формат текущей точки
            assert!(trace.point.contains(':'), "точка должна иметь формат file:line");

            // Проверяем, что предыдущая точка (от throw!) тоже имеет правильный формат
            if let Some(prev_trace) = &trace.prev {
                assert!(prev_trace.point.contains(':'));
            }

            // Дополнительно: убедимся, что количество звеньев увеличилось
            // (внутренняя ошибка имеет свой трейс, внешний добавляет ещё один)
            let mut count = 0;
            let mut current = Some(trace);
            while let Some(t) = current {
                count += 1;
                current = t.prev.as_deref();
            }
            // Ожидаем как минимум два звена: от throw и от try
            assert!(count >= 2, "должно быть как минимум два звена трейса");
        }

        // При no-trace проверяем только наличие ошибки (она не должна изменяться)
        #[cfg(feature = "no-trace")]
        {
            // Макрос просто возвращает ошибку как есть, без trace_point
            // Проверяем только сообщение
            let msg = format!("{}", err.throw);
            assert_eq!(msg, "Inner error");
        }
    }
}