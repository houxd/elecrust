use alloc::{borrow::Cow, string::String};
use core::fmt::{Display, Formatter};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
#[must_use]
pub struct Error(Cow<'static, str>);

impl Error {
    pub fn new(s: &'static str) -> Self {
        Error(Cow::Borrowed(s))
    }
    pub fn new_owned(s: String) -> Self {
        Error(Cow::Owned(s))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error(Cow::Borrowed(s))
    }
}

#[macro_export]
macro_rules! err {
    ($($arg:tt)*) => {
        $crate::error::Error::new_owned($crate::alloc::format!($($arg)*))
    };
}
