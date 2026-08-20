//! Stack trace implementation.
//!
//! Contains the [`Trace`] structure, which is a linked list of trace points.
//! Each point stores a string description (usually `file:line`) and a reference to the previous point.
//!
//! Traces are used inside the [`Error`](crate::Error) type and are automatically
//! added by the `throw!` and `catch!` macros when the `all-trace` feature is enabled,
//! or manually provided when using the `my-trace` feature.

use alloc::boxed::Box;
use alloc::string::String;
use core::fmt::{Display, Formatter};

/// A structure representing a stack trace.
///
/// Consists of the current point and a reference to the previous one (`prev`).
/// When displayed via `Display`, it produces a tree‑like representation.
pub struct Trace {
    /// Description of the current point (e.g., `"src/main.rs:42"` or an arbitrary string).
    pub point: String,
    /// The previous trace point (closer to the root of the error).
    pub prev: Option<Box<Trace>>,
}

/// Helper wrapper for formatted output with indentation.
struct TraceDisplayWrapper<'a> {
    trace: &'a Trace,
    inner: usize,
}

impl Trace {
    /// Creates a new trace object.
    ///
    /// # Parameters
    /// - `point` – a string describing the point.
    /// - `prev` – the previous point (may be `None` for the root).
    ///
    /// # Example
    /// ```
    /// # use dyn_trace_err::trace::Trace;
    /// let trace = Trace::new("foo".to_string(), None);
    /// assert_eq!(trace.point, "foo");
    /// ```
    #[inline(always)]
    pub fn new(point: String, prev: Option<Trace>) -> Self {
        Self {
            point,
            prev: prev.map(Box::new),
        }
    }

    /// Internal formatting helper that respects the nesting level.
    fn display(&self, fmt: &mut Formatter<'_>, inner: usize) -> core::fmt::Result {
        write!(fmt, "| [{}] {}", inner, self.point)?;
        if let Some(prev) = &self.prev {
            write!(fmt, "\n{}", TraceDisplayWrapper {
                trace: prev,
                inner: inner + 1,
            })?;
        }
        Ok(())
    }
}

impl Display for Trace {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.display(f, 0)
    }
}

impl Display for TraceDisplayWrapper<'_> {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        self.trace.display(f, self.inner)
    }
}