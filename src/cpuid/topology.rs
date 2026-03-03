use crate::cpuid::fns::LEAF_16;
use crate::cpuid::{fns, x86_cpuid};

#[derive(Debug, Default)]
pub struct CacheLevel {
    size: u32,
}

#[derive(Debug, Default)]
pub struct Cache {
    l1i: CacheLevel,
    l1d: CacheLevel,
    l2: Option<CacheLevel>,
    l3: Option<CacheLevel>,
}

impl Cache {
    pub fn new(
        l1i: CacheLevel,
        l1d: CacheLevel,
        l2: Option<CacheLevel>,
        l3: Option<CacheLevel>,
    ) -> Cache {
        Cache { l1i, l1d, l2, l3 }
    }
}

#[derive(Debug, Default)]
pub struct Speed {
    pub base: u32,
    pub boost: u32,
    pub measured: bool,
}

impl Speed {
    pub fn detect() -> Self {
        if fns::max_leaf() < LEAF_16 {
            return Speed::default();
        }

        let res = x86_cpuid(LEAF_16);

        Speed {
            base: res.eax,
            boost: res.ebx,
            measured: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct Topology {
    cores: u32,
    threads: u32,
    speed: Speed,
    cache: Option<Cache>,
}

impl Topology {
    pub fn detect() -> Self {
        let threads = fns::logical_cores();
        let cores = 1;
        let speed = Speed::detect();

        Topology::new(cores, threads, speed)
    }
    pub fn new(cores: u32, threads: u32, speed: Speed) -> Topology {
        Topology {
            cores,
            threads,
            speed,
            cache: None,
        }
    }
}
