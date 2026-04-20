//! A 'static' Rust array with 'Vec' conveniences
//!
//! Loosely based on heapless::Vec
use core::fmt;

#[derive(PartialEq, Copy, Clone)]
pub struct StaticVec<T, const N: usize> {
    data: [T; N],
    len: usize,
}

impl<T: fmt::Debug, const N: usize> fmt::Debug for StaticVec<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.data.iter().take(self.len))
            .finish()
    }
}

impl<T: fmt::Display, const N: usize> fmt::Display for StaticVec<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.data.iter().take(self.len);
        if let Some(first) = iter.next() {
            write!(f, "{}", first)?;
            for item in iter {
                write!(f, "{}", item)?;
            }
        }
        Ok(())
    }
}

impl<T: Default, const N: usize> StaticVec<T, N> {
    pub fn new() -> Self {
        Self {
            data: core::array::from_fn(|_| T::default()),
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) {
        if self.len >= N {
            panic!("StaticVec already full");
        }
        self.data[self.len] = value;
        self.len += 1;
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data[..self.len]
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter().take(self.len)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut().take(self.len)
    }

    pub fn first(&self) -> Option<&T> {
        self.data.first()
    }

    pub fn last(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            Some(&self.data[self.len - 1])
        }
    }
}

impl<T: Default, const N: usize> Default for StaticVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Default, const N: usize> IntoIterator for StaticVec<T, N> {
    type Item = T;
    type IntoIter = IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            data: self.data,
            len: self.len,
            pos: 0,
        }
    }
}

pub struct IntoIter<T, const N: usize> {
    data: [T; N],
    len: usize,
    pos: usize,
}

impl<T: Default, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.len {
            let item = core::mem::take(&mut self.data[self.pos]);
            self.pos += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len - self.pos;
        (remaining, Some(remaining))
    }
}

impl<T: Default, const N: usize> ExactSizeIterator for IntoIter<T, N> {}

pub type FeatureList = StaticVec<&'static str, 64>;
