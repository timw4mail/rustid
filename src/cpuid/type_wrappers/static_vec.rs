#[derive(Debug, PartialEq, Copy, Clone)]
pub struct StaticVec<T, const N: usize> {
    data: [T; N],
    len: usize,
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
    type IntoIter = core::array::IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

pub type FeatureList = StaticVec<&'static str, 64>;
