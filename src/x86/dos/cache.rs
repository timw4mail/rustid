#![cfg(dos_real)]

use crate::common::cache::Cache;

impl Cache {
    pub fn detect() -> Option<Self> {
        None
    }
}
