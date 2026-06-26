//! BSD-specific ARM CPU detection (NetBSD, FreeBSD, OpenBSD).
//!
//! On NetBSD, the MIDR is read via `sysctl machdep.cpu_id`. On other BSDs,
//! inline assembly is used instead. NetBSD does not expose per-feature sysctl
//! keys, so `TArmFeatures` uses the trait's default (false) for all features.

use super::OsCpuInfo;
use crate::arm::brand::Vendor;
use crate::arm::micro_arch::*;
use crate::arm::{ArmFeatures, TArmFeatures};
use crate::common::DataSource;
use std::collections::{BTreeMap, HashSet};

/// Reads MIDR values, preferring `sysctl machdep.cpu_id` (NetBSD/OpenBSD)
/// and falling back to inline asm on other BSDs.
fn get_bsd_midrs() -> (Vec<usize>, DataSource) {
    if let Some(val) = crate::common::get_sysctl_value("machdep.cpu_id") {
        if let Ok(midr) = val.trim().parse::<usize>() {
            return (vec![midr], DataSource::Sysctrl("machdep.cpu_id"));
        }
    }

    let mut midrs = Vec::new();
    #[cfg(not(target_arch = "arm"))]
    if let Some(core_ids) = core_affinity::get_core_ids() {
        for core_id in core_ids {
            core_affinity::set_for_current(core_id);
            midrs.push(crate::arm::get_midr());
        }
    } else {
        midrs.push(crate::arm::get_midr());
    }
    #[cfg(target_arch = "arm")]
    midrs.push(crate::arm::get_midr());

    (midrs, DataSource::CpuLookupTable)
}

/// BSD-specific CPU detection via sysctl and inline asm fallback.
pub fn detect() -> OsCpuInfo {
    let mut raw_midr: HashSet<usize> = HashSet::new();
    let mut midrs: HashSet<Midr> = HashSet::new();
    let mut all_midrs: Vec<Midr> = Vec::new();

    let (bsd_midrs, midr_source) = get_bsd_midrs();
    for m_val in bsd_midrs {
        raw_midr.insert(m_val);
        let midr = Midr::new(m_val);
        midrs.insert(midr);
        all_midrs.push(midr);
    }

    let primary_midr = midrs.iter().next().copied().unwrap_or(Midr::default());
    let vendor: String = Vendor::from(primary_midr.implementer).into();
    let cpu_arch = CpuArch::find(
        primary_midr.implementer,
        primary_midr.part,
        primary_midr.variant,
    );
    let cores = super::detect_cores(&all_midrs);

    OsCpuInfo {
        raw_midr,
        midrs,
        vendor,
        cpu_arch,
        cores,
        model: String::new(),
        raw: BTreeMap::new(),
        midr_source,
        features_source: DataSource::DefaultValue,
    }
}

impl TArmFeatures for ArmFeatures {}

/// Returns all detected features as a BTreeMap of category to space-separated features.
pub fn get_all_features() -> BTreeMap<&'static str, String> {
    crate::arm::features::build_feature_map(&BTreeMap::new())
}
