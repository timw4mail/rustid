//! Wrapper object around std::String and a minimal fixed-size string implementation.
//!
//! This helps hide the ugliness of using a fixed-size array for DOS/no_std
use crate::cpuid::type_wrappers::static_vec::StaticVec;
use core::fmt::{self, Debug, Display, Formatter, Write};
use core::ops::Deref;

#[cfg(not(target_os = "none"))]
#[macro_export]
macro_rules! sfmt {
    ($($arg:tt)*) => { Into::<Str<_>>::into(std::format!($($arg)*)) };
}

#[cfg(target_os = "none")]
#[macro_export]
macro_rules! sfmt {
    ($($arg:tt)*) => {
        {
            use $crate::cpuid::type_wrappers::String;
            use core::fmt::Write;
            let mut buf = String::<{ $crate::cpuid::type_wrappers::MAX_FMT_LEN }>::new();
            let _ = buf.write_fmt(core::format_args!($($arg)*));
            $crate::cpuid::type_wrappers::sfmt_into::<{ $crate::cpuid::type_wrappers::MAX_FMT_LEN }, _>(buf)
        }
    };
}

#[cfg(target_os = "none")]
pub fn sfmt_into<const N: usize, const M: usize>(s: String<N>) -> Str<M> {
    let mut result = Str::<M>::new();
    result.push_str(s.as_str());
    result
}

pub const MAX_FMT_LEN: usize = 256;

#[derive(Debug, Clone, PartialEq)]
pub struct String<const N: usize>(StaticVec<u8, N>);

impl<const N: usize> String<N> {
    pub fn new() -> Self {
        Self(StaticVec::new())
    }

    pub fn push(&mut self, c: char) {
        let mut buf = [0u8; 4];
        let bytes = c.encode_utf8(&mut buf);
        self.push_str(bytes);
    }

    pub fn push_str(&mut self, s: &str) {
        let remaining = N - self.0.len();
        let copy_len = s.len().min(remaining);
        for byte in &s.as_bytes()[..copy_len] {
            self.0.push(*byte);
        }
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(self.0.as_slice()).unwrap_or("")
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<const N: usize> Default for String<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Deref for String<N> {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> AsRef<str> for String<N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> core::fmt::Display for String<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<const N: usize> core::fmt::Write for String<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.push_str(s);
        Ok(())
    }
}

impl<const N: usize> From<&str> for String<N> {
    fn from(s: &str) -> Self {
        let mut result = Self::new();
        result.push_str(s);
        result
    }
}

#[cfg(not(target_os = "none"))]
use std::string::String as StdString;

#[derive(Clone, PartialEq)]
pub struct Str<const N: usize> {
    #[cfg(not(target_os = "none"))]
    inner: StdString,

    #[cfg(target_os = "none")]
    inner: String<N>,
}

impl<const N: usize> Default for Str<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Str<N> {
    pub fn new() -> Self {
        Self {
            #[cfg(not(target_os = "none"))]
            inner: StdString::new(),

            #[cfg(target_os = "none")]
            inner: String::new(),
        }
    }
    pub fn push(&mut self, c: char) {
        #[cfg(not(target_os = "none"))]
        self.inner.push(c);

        #[cfg(target_os = "none")]
        self.inner.push(c);
    }
    pub fn push_str(&mut self, s: &str) {
        #[cfg(not(target_os = "none"))]
        self.inner.push_str(s);

        #[cfg(target_os = "none")]
        self.inner.push_str(s);
    }
    pub fn trim(&self) -> &str {
        #[cfg(not(target_os = "none"))]
        return self.inner.trim();

        #[cfg(target_os = "none")]
        {
            let s = self.inner.as_str();
            let start = s.bytes().take_while(|b| b.is_ascii_whitespace()).count();
            let end = s
                .bytes()
                .rposition(|b| b.is_ascii_whitespace())
                .map(|p| p + 1)
                .unwrap_or(s.len());
            &s[start..end.min(s.len())]
        }
    }

    #[cfg(not(target_os = "none"))]
    pub fn replace(&self, from: &str, to: &str) -> Self {
        let s: &str = self.deref();
        let replaced = s.replace(from, to);
        replaced.into()
    }

    #[cfg(target_os = "none")]
    pub fn replace(&self, from: &str, to: &str) -> Self {
        let s = self.inner.as_str();
        let mut result = String::<N>::new();
        let mut last = 0;
        for (idx, _) in s.match_indices(from) {
            result.push_str(&s[last..idx]);
            result.push_str(to);
            last = idx + from.len();
        }
        result.push_str(&s[last..]);
        let mut out = Str::new();
        out.inner = result;
        out
    }
}

#[cfg(not(target_os = "none"))]
impl<const N: usize> From<StdString> for Str<N> {
    fn from(s: StdString) -> Self {
        Self { inner: s }
    }
}

#[cfg(target_os = "none")]
impl<const N: usize> From<String<N>> for Str<N> {
    fn from(s: String<N>) -> Self {
        Self { inner: s }
    }
}

impl<const N: usize> Deref for Str<N> {
    type Target = str;
    fn deref(&self) -> &str {
        #[cfg(not(target_os = "none"))]
        return &self.inner;

        #[cfg(target_os = "none")]
        return self.inner.as_str();
    }
}

#[cfg(target_os = "none")]
impl<const N: usize> Str<N> {
    pub fn from_str(s: &str) -> Self {
        let mut result = Str::<N>::new();
        result.push_str(s);
        result
    }

    pub fn from_str_any<M>(s: &str) -> Self
    where
        M: AsRef<str> + ?Sized,
    {
        let mut result = Str::<N>::new();
        result.push_str(s);
        result
    }
}

#[cfg(target_os = "none")]
pub trait FromStr {
    fn from_str(s: &str) -> Self;
}

#[cfg(target_os = "none")]
impl<const N: usize> FromStr for Str<N> {
    fn from_str(s: &str) -> Self {
        let mut result = Str::<N>::new();
        result.push_str(s);
        result
    }
}

impl<const N: usize> AsRef<str> for Str<N> {
    fn as_ref(&self) -> &str {
        #[cfg(not(target_os = "none"))]
        return &self.inner;

        #[cfg(target_os = "none")]
        return self.inner.as_str();
    }
}

impl<const N: usize> Display for Str<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.deref())
    }
}

impl<const N: usize> Debug for Str<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.deref())
    }
}

impl<const N: usize> Write for Str<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.push_str(s);
        Ok(())
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
