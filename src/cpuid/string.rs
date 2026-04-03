use core::fmt::{self, Display, Formatter};
use core::ops::Deref;

#[cfg(target_os = "none")]
pub use heapless::format;

#[cfg(target_os = "none")]
#[macro_export]
macro_rules! sfmt {
    ($($arg:tt)*) => {
        Into::<Str<_>>::into(heapless::format!($($arg)*).unwrap())
    };
}

#[cfg(not(target_os = "none"))]
#[macro_export]
macro_rules! sfmt {
    ($($arg:tt)*) => { Into::<Str<_>>::into(std::format!($($arg)*)) };
}

#[cfg(target_os = "none")]
use heapless::String as HeaplessString;

#[cfg(not(target_os = "none"))]
use std::string::String as StdString;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Str<const N: usize> {
    #[cfg(not(target_os = "none"))]
    inner: StdString,

    #[cfg(target_os = "none")]
    inner: HeaplessString<N>,
}

impl<const N: usize> Str<N> {
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_os = "none"))]
            inner: StdString::new(),

            #[cfg(target_os = "none")]
            inner: HeaplessString::new(),
        }
    }
    pub fn push(&mut self, c: char) {
        #[cfg(not(target_os = "none"))]
        self.inner.push(c);

        #[cfg(target_os = "none")]
        let _ = self.inner.push(c);
    }
    pub fn push_str(&mut self, s: &str) {
        #[cfg(not(target_os = "none"))]
        self.inner.push_str(s);

        #[cfg(target_os = "none")]
        let _ = self.inner.push_str(s);
    }
    pub fn trim(&self) -> &str {
        self.inner.trim()
    }

    #[cfg(not(target_os = "none"))]
    pub fn replace(&self, from: &str, to: &str) -> Self {
        let s: &str = self.deref();
        let replaced = s.replace(from, to);
        replaced.into()
    }

    #[cfg(target_os = "none")]
    pub fn replace(&self, from: &str, to: &str) -> Self {
        let s: &str = self.deref();
        let mut result = HeaplessString::new();
        let mut last = 0;
        for (idx, _) in s.match_indices(from) {
            let _ = result.push_str(&s[last..idx]);
            let _ = result.push_str(to);
            last = idx + from.len();
        }
        let _ = result.push_str(&s[last..]);
        result.into()
    }
}

#[cfg(not(target_os = "none"))]
impl<const N: usize> From<StdString> for Str<N> {
    fn from(s: StdString) -> Self {
        Self { inner: s }
    }
}

#[cfg(target_os = "none")]
impl<const N: usize> From<HeaplessString<N>> for Str<N> {
    fn from(s: HeaplessString<N>) -> Self {
        Self { inner: s }
    }
}

impl<const N: usize> Deref for Str<N> {
    type Target = str;
    fn deref(&self) -> &str {
        &self.inner
    }
}

impl<const N: usize> AsRef<str> for Str<N> {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl<const N: usize> Display for Str<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<const N: usize> PartialEq<&str> for Str<N> {
    fn eq(&self, other: &&str) -> bool {
        self.deref() == *other
    }
}

impl<const N: usize> PartialEq<str> for Str<N> {
    fn eq(&self, other: &str) -> bool {
        self.deref() == other
    }
}

impl<const N: usize> PartialEq<Str<N>> for &str {
    fn eq(&self, other: &Str<N>) -> bool {
        *self == other.deref()
    }
}

impl<const N: usize> PartialEq<Str<N>> for str {
    fn eq(&self, other: &Str<N>) -> bool {
        self == other.deref()
    }
}

impl<const N: usize> From<&str> for Str<N> {
    fn from(s: &str) -> Self {
        let mut str = Str::new();
        str.push_str(s);

        str
    }
}
