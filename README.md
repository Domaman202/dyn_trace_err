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
dyn_trace_err = "0.2.0"
```

By default the `"all-trace"` feature is enabled. To select a different mode, specify it explicitly:

```toml
[dependencies.dyn_trace_err]
version = "0.2.0"
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
pub struct Error<T: IThrowable + ?Sized> {
    throw: Box<T>,
    #[cfg(not(feature = "no-trace"))]
    trace: trace::Trace,
}
```

**Formatting:**
- `Display` (`{}`) – prints **only the error message** (from the innermost error payload).  
  This is suitable for user‑facing messages.
- `Debug` (`{:?}`) – prints the **full error chain** with all causes and stack traces.  
  Use this for logging or debugging.

#### Methods

- `throwable(&self) -> &T` – returns a reference to the error payload (the type implementing `IThrowable`). This allows you to access additional data stored in your custom error type.

### `IThrowable`

The trait that your error types must implement:

```rust
pub trait IThrowable: Display {
    fn cause(&self) -> &Option<Box<Error<dyn IThrowable>>>;
}
```

The `cause` method returns a reference to the nested error (the cause), allowing chains of errors.

### `Trace`

A structure representing the call stack:

```rust
pub struct Trace {
    pub point: String,
    pub prev: Option<Box<Trace>>,
}
```

Implements `Display` for a pretty tree‑like output (used only in `Debug` formatting of `Error`).

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

### `throw_display!`

Creates an error from any type that implements `Display`, storing it as `DisplayableException`. Useful when you already have an error type that is `Display` but you don’t want to implement `IThrowable` manually.

**Available forms:**

- `throw_display!($expr)` – just a displayable value.
- `throw_display!($expr, $cause)` – displayable value and a cause.
- (for `all-trace` / `my-trace`) `throw_display!($expr, $cause, $trace)` – with an explicit trace.

In `no-trace` mode only the first two forms are available.

---

## 📝 Built‑in Error Types

### `StringException`

Stores a string message and an optional cause.

```rust
let err = StringException::new("Something went wrong".to_string(), None);
throw!(err);   // with all-trace it will add an automatic trace
```

The `throw_string!` macro uses this type.

### `DisplayableException`

Wraps any type that implements `Display`. Useful when you want to throw an error that is not a string but you don’t want to implement `IThrowable`.

```rust
use dyn_trace_err::throw_display;
use std::fmt;

#[derive(Debug)]
enum MyError {
    Foo,
    Bar,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyError::Foo => write!(f, "Foo error"),
            MyError::Bar => write!(f, "Bar error"),
        }
    }
}

throw_display!(MyError::Foo); // creates an error with that displayable value
```

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

**Output with `{}` (Display) – only the error message:**
```
All ok! Value: 21
Error! Value is one!
```

**Output with `{:?}` (Debug) – full chain and traces:**
```
[0] Error! Value is one!
| [0] src/main.rs:39
| [1] src/main.rs:34
| [2] src/main.rs:28
```

> **Note:** To see the full trace, use `println!("{:?}", err);`.

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

**Display output (only messages):**
```
All ok! Value: 12
Error! Value is one!
```

**Debug output (full chain):**
```
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

**Display output (only messages, no traces):**
```
All ok! Value: 33
Error! Value not formatted!
```

**Debug output (still shows the cause chain because `Debug` always includes causes):**
```
[0] Error! Value not formatted!
[1] Error! Value is negative!
```

---

### 4. Using `throw_display!` with a custom enum

```rust
use dyn_trace_err::{throw_display, catch, Error};
use std::fmt;

#[derive(Debug)]
enum MyAppError {
    InvalidInput,
    NetworkFailure,
}

impl fmt::Display for MyAppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyAppError::InvalidInput => write!(f, "Invalid input provided"),
            MyAppError::NetworkFailure => write!(f, "Network failure occurred"),
        }
    }
}

fn parse_input() -> Result<(), Error> {
    // ... some logic ...
    throw_display!(MyAppError::InvalidInput)
}

fn fetch_data() -> Result<(), Error> {
    match parse_input() {
        Ok(()) => Ok(()),
        Err(e) => throw_display!(MyAppError::NetworkFailure, Some(e)),
    }
}

fn main() -> Result<(), Error> {
    catch!(fetch_data());
    Ok(())
}
```

---

### 5. Accessing error data via `throwable()`

Sometimes you need to extract additional data from the error. The `throwable()` method returns a reference to the payload, and you can call your own methods.

```rust
use dyn_trace_err::{Error, IThrowable, throw, catch, throw_string};
use std::fmt;

#[derive(Debug)]
struct MyError {
    code: u32,
    message: String,
    cause: Option<Box<Error<dyn IThrowable>>>,
}

impl IThrowable for MyError {
    fn cause(&self) -> &Option<Box<Error<dyn IThrowable>>> {
        &self.cause
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Code {}: {}", self.code, self.message)
    }
}

impl MyError {
    fn code(&self) -> u32 {
        self.code
    }
}

fn fail() -> Result<(), Error<dyn IThrowable>> {
    let err = MyError {
        code: 404,
        message: "Not found".to_string(),
        cause: None,
    };
    throw!(Box::new(err));
}

fn main() -> Result<(), Error<dyn IThrowable>> {
    let result = fail();
    if let Err(e) = result {
        let my_err = e.throwable();
        println!("Error code: {}", my_err.code());
        println!("Full error: {:?}", e); // Debug shows all details
    }
    Ok(())
}
```

---

### 6. Creating a custom error type with fields and methods

You can implement `IThrowable` for your own type, adding any fields and methods. This allows you to carry contextual information along with the error.

```rust
use dyn_trace_err::{Error, IThrowable, throw, throw_string};
use std::fmt;

enum MyErrorVariant {
    ErrorFoo(i32),
    ErrorBar(f32),
}

impl fmt::Display for MyErrorVariant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MyErrorVariant::ErrorFoo(v) => write!(f, "Foo: {}", v),
            MyErrorVariant::ErrorBar(v) => write!(f, "Bar: {}", v),
        }
    }
}

impl IThrowable for MyErrorVariant {
    fn cause(&self) -> &Option<Box<Error<dyn IThrowable>>> {
        &None  // no nested cause in this example
    }
}

impl MyErrorVariant {
    pub fn value(&self) -> f32 {
        match self {
            MyErrorVariant::ErrorFoo(v) => *v as f32,
            MyErrorVariant::ErrorBar(v) => *v,
        }
    }
}

fn bar() -> Result<(), Error<MyErrorVariant>> {
    throw!(Box::new(MyErrorVariant::ErrorBar(21.777)));
}

fn foo() -> Result<(), Error<MyErrorVariant>> {
    throw!(Box::new(MyErrorVariant::ErrorFoo(12)));
}

fn main() -> Result<(), Error<dyn IThrowable>> {
    let foo_err = foo().unwrap_err();
    let bar_err = bar().unwrap_err();
    let sum = foo_err.throwable().value() + bar_err.throwable().value();
    throw_string!(format!("foo({}) + bar({}) = {}", foo_err.throwable().value(), bar_err.throwable().value(), sum));
}
```

---

## 🧩 Creating Your Own Error Type (general case)

If the built‑in types are not sufficient, implement the `IThrowable` trait for your own type:

```rust
use dyn_trace_err::{IThrowable, Error};
use core::fmt;

struct MyError {
    details: String,
    cause: Option<Box<Error<dyn IThrowable>>>,
}

impl IThrowable for MyError {
    fn cause(&self) -> &Option<Box<Error<dyn IThrowable>>> {
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
throw!(Box::new(err)); // with all-trace it will add an automatic trace
```

---

## 📄 License

This library is distributed under the **MIT** license. See the [LICENSE](LICENSE) file for details.

---

## 🤝 Contributing

If you find a bug or have a suggestion, please open an issue or submit a pull request. Contributions are welcome!