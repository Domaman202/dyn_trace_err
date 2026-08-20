//! Example usage of the `dyn_trace_err` library.
//!
//! This file shows:
//! - custom tracing with explicit `Trace` objects (`custom_traced`),
//! - automatic tracing (`traced`),
//! - error chaining with `cause` (`caused`),
//! - `throw_string!` macro (`string_exception`),
//! - and `throw_display!` macro (`display_exception`).

mod custom_traced {
    use dyn_trace_err::{throw_string, Error, catch};
    use dyn_trace_err::trace::Trace;

    fn bar(value: i32) -> Result<String, Error> {
        if value == 1 { throw_string!("Error! Value is one!", None, Trace::new("bar".to_string(), None)) }
        Ok(format!("All ok! Value: {}", value))
    }

    fn foo(value: i32) -> Result<String, Error> {
        if value == 0 { throw_string!("Error! Value is zero!", None, Trace::new("foo".to_string(), None)) }
        Ok(catch!(bar(value), |prev| Trace::new("foo".to_string(), Some(prev))))
    }

    pub fn test() -> Result<(), Error> {
        println!("{}", catch!(foo(12), |prev| Trace::new("test".to_string(), Some(prev))));
        println!("{}", catch!(foo(1), |prev| Trace::new("test".to_string(), Some(prev))));
        Ok(())
    }
}

mod traced {
    use dyn_trace_err::{throw_string, Error, catch};

    fn bar(value: i32) -> Result<String, Error> {
        if value == 1 { throw_string!("Error! Value is one!") }
        Ok(format!("All ok! Value: {}", value))
    }

    fn foo(value: i32) -> Result<String, Error> {
        if value == 0 { throw_string!("Error! Value is zero!") }
        Ok(catch!(bar(value)))
    }

    pub fn test() -> Result<(), Error> {
        println!("{}", catch!(foo(21)));
        println!("{}", catch!(foo(1)));
        Ok(())
    }
}

mod caused {
    use dyn_trace_err::{throw_string, Error, catch};

    fn bar(value: i32) -> Result<String, Error> {
        if value < 0 { throw_string!("Error! Value is negative!") }
        Ok(format!("All ok! Value: {}", value))
    }

    fn foo(value: i32) -> Result<String, Error> {
        match bar(value) {
            Ok(val) => Ok(val),
            Err(err) => throw_string!("Error! Value not formatted!", Some(err))
        }
    }

    pub fn test() -> Result<(), Error> {
        println!("{}", catch!(foo(33)));
        println!("{}", catch!(foo(-100)));
        Ok(())
    }
}

mod string_exception {
    use dyn_trace_err::{throw_string, Error};

    pub fn test() -> Result<(), Error> {
        throw_string!("Is string exception!")
    }
}

mod display_exception {
    use std::fmt::{Display, Formatter};
    use dyn_trace_err::{throw_display, Error, catch};

    pub enum ErrorVariant {
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

    fn bar() -> Result<(), Error> {
        throw_display!(ErrorVariant::DisplayErrorBar)
    }

    fn foo() -> Result<(), Error> {
        match bar() {
            Ok(()) => Ok(()),
            Err(err) => throw_display!(ErrorVariant::DisplayErrorFoo, Some(err))
        }
    }

    pub fn test() -> Result<(), Error> {
        catch!(foo());
        Ok(())
    }
}

fn main() {
    println!("{}\n", custom_traced::test().unwrap_err().to_string());
    println!("{}\n", traced::test().unwrap_err().to_string());
    println!("{}\n", caused::test().unwrap_err().to_string());
    println!("{}\n", string_exception::test().unwrap_err().to_string());
    println!("{}\n", display_exception::test().unwrap_err().to_string());
}
