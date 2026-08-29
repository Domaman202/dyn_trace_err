//! Example usage of the `dyn_trace_err` library.
//!
//! This file shows:
//! - custom tracing with explicit `Trace` objects (`custom_traced`),
//! - automatic tracing (`traced`),
//! - error chaining with `cause` (`caused`),
//! - `throw_string!` macro (`string_exception`),
//! - and `throw_display!` macro (`display_exception`).
//! - custom error type with fields and methods (`custom_throwable`).
//!
//! At the end, errors are printed in two formats:
//! - `Display` (with `{}`) – shows only the error message.
//! - `Debug`   (with `{:?}`) – shows the full chain including causes and traces.

mod custom_traced {
    use dyn_trace_err::trace::Trace;
    use dyn_trace_err::{Error, IThrowable, catch, throw_string};

    fn bar(value: i32) -> Result<String, Error<dyn IThrowable>> {
        if value == 1 { throw_string!("Error! Value is one!", None, Trace::new("bar".to_string(), None)) }
        Ok(format!("All ok! Value: {}", value))
    }

    fn foo(value: i32) -> Result<String, Error<dyn IThrowable>> {
        if value == 0 { throw_string!("Error! Value is zero!", None, Trace::new("foo".to_string(), None)) }
        Ok(catch!(bar(value), |prev| Trace::new("foo".to_string(), Some(prev))))
    }

    pub fn test() -> Result<(), Error<dyn IThrowable>> {
        println!("{}", catch!(foo(12), |prev| Trace::new("test".to_string(), Some(prev))));
        println!("{}", catch!(foo(1), |prev| Trace::new("test".to_string(), Some(prev))));
        Ok(())
    }
}

mod traced {
    use dyn_trace_err::{Error, IThrowable, catch, throw_string};

    fn bar(value: i32) -> Result<String, Error<dyn IThrowable>> {
        if value == 1 { throw_string!("Error! Value is one!") }
        Ok(format!("All ok! Value: {}", value))
    }

    fn foo(value: i32) -> Result<String, Error<dyn IThrowable>> {
        if value == 0 { throw_string!("Error! Value is zero!") }
        Ok(catch!(bar(value)))
    }

    pub fn test() -> Result<(), Error<dyn IThrowable>> {
        println!("{}", catch!(foo(21)));
        println!("{}", catch!(foo(1)));
        Ok(())
    }
}

mod caused {
    use dyn_trace_err::{Error, IThrowable, catch, throw_string};

    fn bar(value: i32) -> Result<String, Error<dyn IThrowable>> {
        if value < 0 { throw_string!("Error! Value is negative!") }
        Ok(format!("All ok! Value: {}", value))
    }

    fn foo(value: i32) -> Result<String, Error<dyn IThrowable>> {
        match bar(value) {
            Ok(val) => Ok(val),
            Err(err) => throw_string!("Error! Value not formatted!", Some(err))
        }
    }

    pub fn test() -> Result<(), Error<dyn IThrowable>> {
        println!("{}", catch!(foo(33)));
        println!("{}", catch!(foo(-100)));
        Ok(())
    }
}

mod string_exception {
    use dyn_trace_err::{Error, IThrowable, throw_string};

    pub fn test() -> Result<(), Error<dyn IThrowable>> {
        throw_string!("Is string exception!")
    }
}

mod formattable_exception {
    use dyn_trace_err::{catch, throw_formattable, Error, IThrowable};
    use std::fmt::{Debug, Display, Formatter};
    use dyn_trace_err::r#impl::Formattable;

    enum ErrorVariant {
        DisplayErrorFoo,
        DisplayErrorBar
    }

    impl Display for ErrorVariant {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
            let text =
                match self {
                    ErrorVariant::DisplayErrorFoo => "Is display exception - Foo!",
                    ErrorVariant::DisplayErrorBar => "Is display exception - Bar!",
                };
            f.write_str(text)
        }
    }

    impl Debug for ErrorVariant {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
            let text =
                match self {
                    ErrorVariant::DisplayErrorFoo => "Is debug exception - Foo!",
                    ErrorVariant::DisplayErrorBar => "Is debug exception - Bar!",
                };
            f.write_str(text)
        }
    }

    impl Formattable for ErrorVariant {
    }

    fn bar() -> Result<(), Error<dyn IThrowable>> {
        throw_formattable!(ErrorVariant::DisplayErrorBar)
    }

    fn foo() -> Result<(), Error<dyn IThrowable>> {
        match bar() {
            Ok(()) => Ok(()),
            Err(err) => throw_formattable!(ErrorVariant::DisplayErrorFoo, Some(err))
        }
    }

    pub fn test() -> Result<(), Error<dyn IThrowable>> {
        catch!(foo());
        Ok(())
    }
}

mod custom_throwable {
    use std::fmt::{Debug, Display, Formatter};
    use dyn_trace_err::{Error, IThrowable, throw};

    enum ErrorVariant {
        ErrorFoo(i32),
        ErrorBar(f32),
        ErrorSum(f32, f32, f32)
    }

    impl Display for ErrorVariant {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                ErrorVariant::ErrorSum(foo, bar, sum) => f.write_fmt(format_args!("[Display] foo({}) + bar({}) = {}", foo, bar, sum)),
                _ => unimplemented!()
            }
        }
    }

    impl Debug for ErrorVariant {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                ErrorVariant::ErrorSum(foo, bar, sum) => f.write_fmt(format_args!("[Debug] foo({}) + bar({}) = {}", foo, bar, sum)),
                _ => unimplemented!()
            }
        }
    }

    impl IThrowable for ErrorVariant {
        fn cause(&self) -> &Option<Box<Error<dyn IThrowable>>> {
            &None
        }
    }

    impl ErrorVariant {
        pub fn value(&self) -> f32 {
            match self {
                ErrorVariant::ErrorFoo(value) => *value as f32,
                ErrorVariant::ErrorBar(value) => *value,
                ErrorVariant::ErrorSum(..) => unimplemented!()
            }
        }
    }

    fn bar() -> Result<(), Error<ErrorVariant>> {
        throw!(Box::new(ErrorVariant::ErrorBar(21.777)));
    }

    fn foo() -> Result<(), Error<ErrorVariant>> {
        throw!(Box::new(ErrorVariant::ErrorFoo(12)));
    }

    pub fn test() -> Result<(), Error<dyn IThrowable>> {
        let foo = foo().unwrap_err().throwable().value();
        let bar = bar().unwrap_err().throwable().value();
        throw!(Box::new(ErrorVariant::ErrorSum(foo, bar, foo + bar)));
    }
}

fn main() {
    println!("\n =========== [Display] =========== \n");
    println!("{}\n", custom_traced::test().unwrap_err());
    println!("{}\n", traced::test().unwrap_err());
    println!("{}\n", caused::test().unwrap_err());
    println!("{}\n", string_exception::test().unwrap_err());
    println!("{}\n", formattable_exception::test().unwrap_err());
    println!("{}\n", custom_throwable::test().unwrap_err());
    println!("\n ============ [Debug] ============ \n");
    println!("{:?}\n", custom_traced::test().unwrap_err());
    println!("{:?}\n", traced::test().unwrap_err());
    println!("{:?}\n", caused::test().unwrap_err());
    println!("{:?}\n", string_exception::test().unwrap_err());
    println!("{:?}\n", formattable_exception::test().unwrap_err());
    println!("{:?}\n", custom_throwable::test().unwrap_err());
}