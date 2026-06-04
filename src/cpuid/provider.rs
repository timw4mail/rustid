use super::{Cpuid, real_x86_cpuid_count};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{LazyLock, RwLock};

#[derive(Debug, PartialEq)]
pub enum CpuidInfoSource {
    DumpFile,
    Cpu,
}

/// Trait abstracting the CPUID provider, allowing for mocking in tests.
///
/// This trait enables dependency injection of CPUID providers,
/// which is useful for testing without requiring real x86 hardware.
pub trait CpuidProvider: Send + Sync {
    /// Execute CPUID with the given leaf and sub-leaf.
    fn cpuid_count(&self, leaf: u32, sub_leaf: u32) -> Cpuid;

    /// Where did the CPUID info come from?
    fn info_source(&self) -> CpuidInfoSource;
}

/// Real CPUID provider that executes the CPUID instruction on x86 hardware.
pub struct RealCpuid;

impl CpuidProvider for RealCpuid {
    fn cpuid_count(&self, leaf: u32, sub_leaf: u32) -> Cpuid {
        real_x86_cpuid_count(leaf, sub_leaf)
    }

    fn info_source(&self) -> CpuidInfoSource {
        CpuidInfoSource::Cpu
    }
}

pub(crate) static PROVIDER: LazyLock<RwLock<Box<dyn CpuidProvider + Send + Sync>>> =
    LazyLock::new(|| RwLock::new(Box::new(RealCpuid)));

thread_local! {
    static THREAD_PROVIDER: RefCell<Option<Box<dyn CpuidProvider>>> = const { RefCell::new(None) };
}

/// Sets a custom CPUID provider for the current thread (used primarily for testing).
pub fn set_cpuid_provider<P: CpuidProvider + Send + Sync + 'static>(provider: P) {
    THREAD_PROVIDER.with(|p| {
        *p.borrow_mut() = Some(Box::new(provider));
    });
}

/// Resets the CPUID provider for the current thread.
pub fn reset_cpuid_provider() {
    THREAD_PROVIDER.with(|p| {
        *p.borrow_mut() = None;
    });
}

/// Sets a custom global CPUID provider.
pub fn set_global_cpuid_provider<P: CpuidProvider + Send + Sync + 'static>(provider: P) {
    let mut p = PROVIDER
        .write()
        .expect("Failed to get CPUID Provider RefCell for writing");
    *p = Box::new(provider);
}

/// Resets the global CPUID provider to the real implementation.
pub fn reset_global_cpuid_provider() {
    let mut p = PROVIDER
        .write()
        .expect("Failed to get CPUID Provider RefCell for writing");
    *p = Box::new(RealCpuid);
}

pub(crate) fn cpuid_count(leaf: u32, sub_leaf: u32) -> Cpuid {
    THREAD_PROVIDER.with(|tp| {
        if let Some(p) = tp.borrow().as_ref() {
            return p.cpuid_count(leaf, sub_leaf);
        }
        PROVIDER
            .read()
            .expect("Failed to get CPUID Provider")
            .cpuid_count(leaf, sub_leaf)
    })
}

pub(crate) fn info_source() -> CpuidInfoSource {
    THREAD_PROVIDER.with(|tp| {
        if let Some(p) = tp.borrow().as_ref() {
            return p.info_source();
        }
        PROVIDER
            .read()
            .expect("Failed to get CPUID Provider")
            .info_source()
    })
}

// ----------------------------------------------------------------------------
// CPUID Dump Parser
// ----------------------------------------------------------------------------

thread_local! {
    static CURRENT_DUMP_CPU: Cell<usize> = const { Cell::new(0) };
    static DUMP_CPU_COUNT: Cell<usize> = const { Cell::new(0) };
}

/// Sets the active CPU context index for per-CPU dump providers.
pub fn set_dump_cpu(idx: usize) {
    CURRENT_DUMP_CPU.set(idx);
}

/// Returns the number of CPU contexts in the current dump provider.
pub fn dump_cpu_count() -> usize {
    DUMP_CPU_COUNT.with(|c| c.get())
}

#[derive(Debug, Clone)]
pub struct CpuDump {
    pub cpus: Vec<HashMap<(u32, u32), Cpuid>>,
}

impl Default for CpuDump {
    fn default() -> Self {
        let cpus = vec![HashMap::new()];
        DUMP_CPU_COUNT.with(|c| c.set(cpus.len()));
        Self { cpus }
    }
}

impl CpuDump {
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Self {
        let contents = fs::read_to_string(path).expect("Failed to read dump file");
        let mut cpus: Vec<HashMap<(u32, u32), Cpuid>> = Vec::new();
        let mut current: Option<HashMap<(u32, u32), Cpuid>> = None;

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with("CPU ") {
                if let Some(map) = current.take() {
                    cpus.push(map);
                }
                current = Some(HashMap::new());
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let leaf_str = parts[0].trim_start_matches("0x");
            let subleaf_str = parts[1].trim_end_matches(':').trim_start_matches("0x");

            let Ok(leaf) = u32::from_str_radix(leaf_str, 16) else {
                continue;
            };
            let Ok(sub_leaf) = u32::from_str_radix(subleaf_str, 16) else {
                continue;
            };

            let mut eax = 0u32;
            let mut ebx = 0u32;
            let mut ecx = 0u32;
            let mut edx = 0u32;

            for part in &parts[2..] {
                let reg_val = part.trim_end_matches(',');
                if let Some(val) = reg_val.strip_prefix("eax=") {
                    eax = u32::from_str_radix(val.trim_start_matches("0x"), 16).unwrap_or(0);
                } else if let Some(val) = reg_val.strip_prefix("ebx=") {
                    ebx = u32::from_str_radix(val.trim_start_matches("0x"), 16).unwrap_or(0);
                } else if let Some(val) = reg_val.strip_prefix("ecx=") {
                    ecx = u32::from_str_radix(val.trim_start_matches("0x"), 16).unwrap_or(0);
                } else if let Some(val) = reg_val.strip_prefix("edx=") {
                    edx = u32::from_str_radix(val.trim_start_matches("0x"), 16).unwrap_or(0);
                }
            }

            if let Some(ref mut map) = current {
                map.insert((leaf, sub_leaf), Cpuid { eax, ebx, ecx, edx });
            }
        }

        if let Some(map) = current {
            cpus.push(map);
        }

        // If no CPU sections were found, treat entire file as a single context
        if cpus.is_empty() {
            let mut map = HashMap::new();
            for line in contents.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 2 {
                    continue;
                }
                let leaf_str = parts[0].trim_start_matches("0x");
                let subleaf_str = parts[1].trim_end_matches(':').trim_start_matches("0x");
                let Ok(leaf) = u32::from_str_radix(leaf_str, 16) else {
                    continue;
                };
                let Ok(sub_leaf) = u32::from_str_radix(subleaf_str, 16) else {
                    continue;
                };
                let mut eax = 0u32;
                let mut ebx = 0u32;
                let mut ecx = 0u32;
                let mut edx = 0u32;
                for part in &parts[2..] {
                    let reg_val = part.trim_end_matches(',');
                    if let Some(val) = reg_val.strip_prefix("eax=") {
                        eax = u32::from_str_radix(val.trim_start_matches("0x"), 16).unwrap_or(0);
                    } else if let Some(val) = reg_val.strip_prefix("ebx=") {
                        ebx = u32::from_str_radix(val.trim_start_matches("0x"), 16).unwrap_or(0);
                    } else if let Some(val) = reg_val.strip_prefix("ecx=") {
                        ecx = u32::from_str_radix(val.trim_start_matches("0x"), 16).unwrap_or(0);
                    } else if let Some(val) = reg_val.strip_prefix("edx=") {
                        edx = u32::from_str_radix(val.trim_start_matches("0x"), 16).unwrap_or(0);
                    }
                }
                map.insert((leaf, sub_leaf), Cpuid { eax, ebx, ecx, edx });
            }
            cpus.push(map);
        }

        DUMP_CPU_COUNT.with(|c| c.set(cpus.len()));
        CpuDump { cpus }
    }

    #[must_use]
    pub fn get(&self, leaf: u32, sub_leaf: u32) -> Cpuid {
        let idx = CURRENT_DUMP_CPU.with(|c| c.get());
        self.cpus
            .get(idx)
            .and_then(|map| map.get(&(leaf, sub_leaf)))
            .copied()
            .unwrap_or_default()
    }
}

impl CpuidProvider for CpuDump {
    fn cpuid_count(&self, leaf: u32, sub_leaf: u32) -> Cpuid {
        self.get(leaf, sub_leaf)
    }

    fn info_source(&self) -> CpuidInfoSource {
        CpuidInfoSource::DumpFile
    }
}
