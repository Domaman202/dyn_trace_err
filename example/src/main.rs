use dyn_trace_err::{throw_string, Error, catch};

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
        println!("{}", catch!(foo(12)));
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
            Err(err) => throw_string!("Error! Value not formatted!", err)
        }
    }

    pub fn test() -> Result<(), Error> {
        println!("{}", catch!(foo(21)));
        println!("{}", catch!(foo(-100)));
        Ok(())
    }
}

fn main() {
    println!("{}\n", traced::test().unwrap_err().to_string());
    println!("{}\n", caused::test().unwrap_err().to_string());
}
