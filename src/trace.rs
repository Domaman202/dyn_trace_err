use alloc::boxed::Box;
use alloc::string::String;
use core::fmt::{Display, Formatter};

pub struct Trace {
    pub point: String,
    pub prev: Option<Box<Trace>>
}

struct TraceDisplayWrapper<'a> {
    trace: &'a Trace,
    inner: usize
}

impl Trace {
    #[inline(always)]
    pub fn new(point: String, prev: Option<Trace>) -> Self {
        Self { point, prev: prev.map(Box::new) }
    }

    fn display(&self, fmt: &mut Formatter<'_>, inner: usize) -> core::fmt::Result {
        write!(fmt, "| [{}] {}", inner, self.point)?;
        if let Some(prev) = &self.prev {
            write!(fmt, "\n{}", TraceDisplayWrapper { trace: prev, inner: inner + 1 })?;
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