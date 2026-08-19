#![cfg(dos)]

use crate::common::cache::Cache;

impl Cache {
    pub fn detect() -> Option<Self> {
        None
    }
}
