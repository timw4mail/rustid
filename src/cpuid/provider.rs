use super::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{LazyLock, RwLock};

/// Trait abstracting the CPUID provider, allowing for mocking in tests.
///
/// This trait enables dependency injection of CPUID providers,
/// which is useful for testing without requiring real x86 hardware.
pub trait CpuidProvider: Send + Sync {
    /// Execute CPUID with the given leaf and sub-leaf.
    fn cpuid_count(&self, leaf: u32, sub_leaf: u32) -> Cpuid;
}

/// Real CPUID provider that executes the CPUID instruction on x86 hardware.
pub struct RealCpuid;

impl CpuidProvider for RealCpuid {
    fn cpuid_count(&self, leaf: u32, sub_leaf: u32) -> Cpuid {
        real_x86_cpuid_count(leaf, sub_leaf)
    }
}

pub(crate) static PROVIDER: LazyLock<RwLock<Box<dyn CpuidProvider + Send + Sync>>> =
    LazyLock::new(|| RwLock::new(Box::new(RealCpuid) as Box<dyn CpuidProvider + Send + Sync>));

/// Sets a custom CPUID provider (used primarily for testing).
pub fn set_cpuid_provider<P: CpuidProvider + Send + Sync + 'static>(provider: P) {
    let mut p = PROVIDER.write().unwrap();
    *p = Box::new(provider);
}

/// Resets the CPUID provider to the real implementation.
pub fn reset_cpuid_provider() {
    let mut p = PROVIDER.write().unwrap();
    *p = Box::new(RealCpuid);
}

// ----------------------------------------------------------------------------
// CPUID Dump Parser
// ----------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
pub struct CpuDump {
    pub leaves: HashMap<(u32, u32), Cpuid>,
}

impl CpuDump {
    pub fn parse_file<P: AsRef<Path>>(path: P) -> Self {
        let contents = fs::read_to_string(path).expect("Failed to read dump file");
        let mut leaves: HashMap<(u32, u32), Cpuid> = HashMap::new();

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("CPU ") {
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

            leaves.insert((leaf, sub_leaf), Cpuid { eax, ebx, ecx, edx });
        }

        CpuDump { leaves }
    }

    pub fn get(&self, leaf: u32, sub_leaf: u32) -> Cpuid {
        self.leaves
            .get(&(leaf, sub_leaf))
            .copied()
            .unwrap_or_default()
    }
}
