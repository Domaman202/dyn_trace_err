use alloc::boxed::Box;
use alloc::string::String;
use core::fmt::{Display, Formatter};
use crate::{Error, IThrowable};

pub struct StringException {
    message: String,
    cause: Option<Box<Error>>
}

impl StringException {
    #[inline(always)]
    pub fn new(message: String, cause: Option<Error>) -> Box<dyn IThrowable> {
        Box::new(Self { message, cause: cause.map(Box::new) })
    }
}

impl IThrowable for StringException {
    #[inline(always)]
    fn cause(&self) -> &Option<Box<Error>> where Self: Sized {
        &self.cause
    }
}

impl Display for StringException {
    #[inline(always)]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.message)
    }
}

#[macro_export]
macro_rules! throw_string {
    ($msg:expr) => {
        $crate::throw!($crate::r#impl::StringException::new($msg.to_string(), None));
    };
    ($msg:expr, $cause:expr) => {
        $crate::throw!($crate::r#impl::StringException::new($msg.to_string(), Some($cause)));
    };
}