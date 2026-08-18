use dyn_trace_err::{throw_string, Error, catch};

mod custom_traced {
    use dyn_trace_err::trace::Trace;
    use super::*;

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
    use super::*;

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
    use super::*;

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

fn main() {
    println!("{}\n", custom_traced::test().unwrap_err().to_string());
    println!("{}\n", traced::test().unwrap_err().to_string());
    println!("{}\n", caused::test().unwrap_err().to_string());
}
