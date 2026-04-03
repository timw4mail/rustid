use core::fmt::{self, Display, Formatter};
use core::ops::Deref;
use core::str::FromStr;

#[cfg(target_os = "none")]
pub use heapless::format;

#[cfg(not(target_os = "none"))]
pub use std::format;

#[cfg(target_os = "none")]
use heapless::String as HeaplessString;

#[cfg(not(target_os = "none"))]
use heapless::String as HeaplessString;

#[cfg(not(target_os = "none"))]
use std::string::String as StdString;

pub trait StringBuffer<const N: usize> {
    fn new() -> Self;
    fn push(&mut self, c: char);
    fn push_str(&mut self, s: &str);
}

#[cfg(target_os = "none")]
impl<const N: usize> StringBuffer<N> for HeaplessString<N> {
    fn new() -> Self {
        HeaplessString::new()
    }
    fn push(&mut self, c: char) {
        let _ = HeaplessString::push(self, c);
    }
    fn push_str(&mut self, s: &str) {
        let _ = self.push_str(s);
    }
}

#[cfg(not(target_os = "none"))]
impl<const N: usize> StringBuffer<N> for StdString {
    fn new() -> Self {
        StdString::new()
    }
    fn push(&mut self, c: char) {
        StdString::push(self, c);
    }
    fn push_str(&mut self, s: &str) {
        StdString::push_str(self, s);
    }
}

#[cfg(not(target_os = "none"))]
#[derive(Debug, Default, Clone)]
pub struct Str<const N: usize> {
    inner: StdString,
}

#[cfg(not(target_os = "none"))]
impl<const N: usize> Str<N> {
    pub fn new() -> Self {
        Self {
            inner: StdString::new(),
        }
    }
    pub fn push(&mut self, c: char) {
        self.inner.push(c);
    }
    pub fn push_str(&mut self, s: &str) {
        self.inner.push_str(s);
    }
    pub fn trim(&self) -> &str {
        self.inner.trim()
    }
}

#[cfg(not(target_os = "none"))]
impl<const N: usize> From<StdString> for Str<N> {
    fn from(s: StdString) -> Self {
        Self { inner: s }
    }
}

#[cfg(target_os = "none")]
#[derive(Debug, Default, Clone)]
pub struct Str<const N: usize> {
    inner: HeaplessString<N>,
}

#[cfg(target_os = "none")]
impl<const N: usize> Str<N> {
    pub fn new() -> Self {
        Self {
            inner: HeaplessString::new(),
        }
    }
    pub fn push(&mut self, c: char) {
        let _ = self.inner.push(c);
    }
    pub fn push_str(&mut self, s: &str) {
        let _ = self.inner.push_str(s);
    }
    pub fn trim(&self) -> &str {
        self.inner.trim()
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

impl<const N: usize> AsRef<[u8]> for Str<N> {
    fn as_ref(&self) -> &[u8] {
        self.inner.as_bytes()
    }
}

impl<const N: usize> Display for Str<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl<const N: usize> FromStr for Str<N> {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut this = Self::new();
        this.push_str(s);
        Ok(this)
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

#[cfg(not(target_os = "none"))]
impl<const N: usize> From<HeaplessString<N>> for Str<N> {
    fn from(s: HeaplessString<N>) -> Self {
        let mut this = Self::new();
        this.inner.push_str(&s);
        this
    }
}

impl<const N: usize> PartialEq<Str<N>> for Str<N> {
    fn eq(&self, other: &Str<N>) -> bool {
        self.deref() == other.deref()
    }
}
