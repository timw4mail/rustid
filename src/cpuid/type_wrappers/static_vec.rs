#[derive(Debug, PartialEq)]
pub struct StaticVec<T, const N: usize> {
    data: [T; N],
    len: usize,
}

impl<T, const N: usize> StaticVec<T, N> {
    pub const fn new() -> Self {
        Self {
            data: unsafe { core::mem::zeroed() },
            len: 0,
        }
    }

    pub fn push(&mut self, value: T) -> bool {
        if self.len >= N {
            return false;
        }
        self.data[self.len] = value;
        self.len += 1;
        true
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
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

impl<T: Clone, const N: usize> Clone for StaticVec<T, N> {
    fn clone(&self) -> Self {
        let mut new = Self::new();
        for item in self.iter() {
            let _ = new.push(item.clone());
        }
        new
    }
}

impl<T: Copy, const N: usize> Copy for StaticVec<T, N> {}

impl<T, const N: usize> Default for StaticVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> IntoIterator for StaticVec<T, N> {
    type Item = T;
    type IntoIter = core::array::IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

pub type FeatureList = StaticVec<&'static str, 64>;
