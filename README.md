# dyn_trace_err

**Dynamic errors with a stack trace.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![no_std](https://img.shields.io/badge/no__std-compatible-green.svg)](https://docs.rust-embedded.org)

A flexible error‑handling library for Rust that supports **stack traces**, **nested causes** (`cause`), and works in `no_std` environments. Choose between automatic, custom, or completely disabled tracing via Cargo features.

---

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
dyn_trace_err = "0.1.1"
```

By default the `"all-trace"` feature is enabled. To select a different mode, specify it explicitly:

```toml
[dependencies.dyn_trace_err]
version = "0.1.1"
default-features = false
features = ["my-trace"]   # or "no-trace"
```

---

## ⚙️ Choosing the Tracing Mode

The library provides **exactly three modes**, and you must select **exactly one** (compilation will fail if none or more than one is selected).

| Feature     | Description |
|-------------|-------------|
| `all-trace` | **Automatic tracing**. The `throw!` and `catch!` macros automatically add `file!()` and `line!()` at each new trace point. |
| `my-trace`  | **Custom tracing**. You explicitly pass a `Trace` object to the macros. |
| `no-trace`  | **Tracing disabled**. The `Error` struct does not contain the `trace` field, and macros do not accept trace arguments. Minimal overhead. |

---

## 🧱 Core Types

### `Error`

The main error type returned from functions:

```rust
pub struct Error {
    pub throw: Box<dyn IThrowable>,
    #[cfg(not(feature = "no-trace"))]
    pub trace: trace::Trace,
}
```

Implements `Display` – when printed, it outputs the error message and the entire chain of causes with their traces (if any).

### `IThrowable`

The trait that your error types must implement:

```rust
pub trait IThrowable: Display {
    fn cause(&self) -> &Option<Box<Error>>;
}
```

The `cause` method returns a reference to the nested error (the cause), allowing chains of errors.

### `Trace`

A structure representing the call stack:

```rust
pub struct Trace {
    pub point: String,         // description of the point (e.g., "file:line" or a custom string)
    pub prev: Option<Box<Trace>>,
}
```

Implements `Display` for a pretty tree‑like output.

---

## 🔧 Macros

### `throw!`

Creates an error and immediately returns `Result` with `Err`.

**Variants depending on the feature:**

| Feature    | Available forms |
|------------|-----------------|
| `all-trace` | `throw!($expr)` – automatically adds `file!()` and `line!()`<br>`throw!($expr, $trace)` – with an explicit `Trace` |
| `my-trace`  | `throw!($expr, $trace)` – only with an explicit `Trace` |
| `no-trace`  | `throw!($expr)` – no trace |

### `catch!`

Evaluates an expression that returns a `Result`. On error, it adds a new trace point (or simply propagates the error when `no-trace` is used).

**Variants:**

| Feature    | Available forms |
|------------|-----------------|
| `all-trace` | `catch!($expr)` – automatic `file!()` and `line!()`<br>`catch!($expr, $trace)` – with an explicit `Trace` |
| `my-trace`  | `catch!($expr, $trace)` – only with an explicit `Trace` |
| `no-trace`  | `catch!($expr)` – just propagates the error |

### `throw_string!`

A convenient wrapper that creates an error of type `StringException` (a built‑in `IThrowable` implementation with a string message).

**Available forms:**

- `throw_string!($msg)` – just a message.
- `throw_string!($msg, $cause)` – message and a cause.
- (for `all-trace` / `my-trace`) `throw_string!($msg, $cause, $trace)` – with an explicit trace.

In `no-trace` mode only the first two forms are available.

---

## 📝 Built‑in `StringException`

The library provides a ready‑made `IThrowable` implementation – `StringException`. It stores a string message and an optional cause.

```rust
let err = StringException::new("Something went wrong".to_string(), None);
throw!(err);   // with all-trace it will add an automatic trace
```

The `throw_string!` macro uses this type, so for simple cases you don't need to write your own error types.

---

## 💡 Usage Examples

### 1. Automatic Tracing (`all-trace`)

```rust
use dyn_trace_err::{throw_string, catch, Error};

fn bar(value: i32) -> Result<String, Error> {
    if value == 1 {
        throw_string!("Error! Value is one!");
    }
    Ok(format!("All ok! Value: {}", value))
}

fn foo(value: i32) -> Result<String, Error> {
    if value == 0 {
        throw_string!("Error! Value is zero!");
    }
    Ok(catch!(bar(value)))
}

fn main() -> Result<(), Error> {
    println!("{}", catch!(foo(21))?);
    println!("{}", catch!(foo(1))?); // this will error
    Ok(())
}
```

Output:

```
All ok! Value: 21
[0] Error! Value is one!
| [0] src/main.rs:39
| [1] src/main.rs:34
| [2] src/main.rs:28
```

---

### 2. Custom Tracing (`my-trace`)

Here you create `Trace` objects yourself and pass them to the macros.

```rust
use dyn_trace_err::{throw_string, catch, Error, trace::Trace};

fn bar(value: i32) -> Result<String, Error> {
    if value == 1 {
        throw_string!("Error! Value is one!", None, Trace::new("bar".to_string(), None));
    }
    Ok(format!("All ok! Value: {}", value))
}

fn foo(value: i32) -> Result<String, Error> {
    if value == 0 {
        throw_string!("Error! Value is zero!", None, Trace::new("foo".to_string(), None));
    }
    Ok(catch!(bar(value), |prev| Trace::new("foo".to_string(), Some(prev))))
}

fn main() -> Result<(), Error> {
    println!("{}", catch!(foo(12), |prev| Trace::new("test".to_string(), Some(prev)))?);
    println!("{}", catch!(foo(1), |prev| Trace::new("test".to_string(), Some(prev)))?);
    Ok(())
}
```

Output:

```
All ok! Value: 12
[0] Error! Value is one!
| [0] test
| [1] foo
| [2] bar
```

---

### 3. No Tracing (`no-trace`)

Errors do not contain the `trace` field, `catch!` simply propagates the error. Minimal footprint and maximum performance.

```rust
use dyn_trace_err::{throw_string, catch, Error};

fn bar(value: i32) -> Result<String, Error> {
    if value < 0 {
        throw_string!("Error! Value is negative!");
    }
    Ok(format!("All ok! Value: {}", value))
}

fn foo(value: i32) -> Result<String, Error> {
    match bar(value) {
        Ok(val) => Ok(val),
        Err(err) => throw_string!("Error! Value not formatted!", Some(err)),
    }
}

fn main() -> Result<(), Error> {
    println!("{}", catch!(foo(33))?);
    println!("{}", catch!(foo(-100))?);
    Ok(())
}
```

Output (only the cause chain, no traces):

```
All ok! Value: 33
[0] Error! Value not formatted!
[1] Error! Value is negative!
```

---

## 🧩 Creating Your Own Error Type

If `StringException` is not sufficient, implement the `IThrowable` trait for your own type:

```rust
use dyn_trace_err::{IThrowable, Error};
use core::fmt;

struct MyError {
    details: String,
    cause: Option<Box<Error>>,
}

impl IThrowable for MyError {
    fn cause(&self) -> &Option<Box<Error>> {
        &self.cause
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MyError: {}", self.details)
    }
}
```

Then use `throw!` with an instance:

```rust
let err = MyError { details: "oops".into(), cause: None };
throw!(err); // with all-trace it will add an automatic trace
```

---

## 📄 License

This library is distributed under the **MIT** license. See the [LICENSE](LICENSE) file for details.

---

## 🤝 Contributing

If you find a bug or have a suggestion, please open an issue or submit a pull request. Contributions are welcome!