use super::OsCpuInfo;
use crate::arm::TArmFeatures;
use crate::arm::brand::*;
use crate::arm::micro_arch::*;
use crate::common::get_full_raw_sysctl_map;
use crate::common::{Cache, CacheLevel, CacheType, CoreType, DataSource, Level1Cache, UNK};
use std::collections::{BTreeMap, HashSet};

// ----------------------------------------------------------------------------
// Feature detection via sysctl (text-based, matches existing pattern)
// ----------------------------------------------------------------------------

/// Parses sysctl output for hw.optional.* keys to detect CPU features.
/// All feature names are converted to lowercase for consistency.
pub fn get_features_from_sysctl() -> BTreeMap<String, bool> {
    let mut features: BTreeMap<String, bool> = BTreeMap::new();

    // Run sysctl to get hw.optional and hw.optional.arm keys
    for (key, value) in get_full_raw_sysctl_map() {
        // Only process hw.optional.* keys
        if key.starts_with("hw.optional") {
            // Convert value to bool (1 = true, 0 = false)
            if let Ok(v) = value.parse::<i32>()
                && v == 1
            {
                // Extract feature name from key
                // e.g., "hw.optional.neon" -> "neon"
                // e.g., "hw.optional.arm.FEAT_AES" -> "aes"
                let feature_name = if key.starts_with("hw.optional.arm.FEAT_") {
                    // Remove "hw.optional.arm.FEAT_" prefix
                    let feat = key
                        .strip_prefix("hw.optional.arm.FEAT_")
                        .expect("starts_with guard ensures this matches");
                    feat.to_lowercase()
                } else if key.starts_with("hw.optional.arm.FEAT") {
                    // Handle "hw.optional.arm.FEATXYZ" without underscore
                    let feat = key
                        .strip_prefix("hw.optional.arm.FEAT")
                        .expect("starts_with guard ensures this matches");
                    feat.to_lowercase()
                } else if key.starts_with("hw.optional.arm.") {
                    let feat = key
                        .strip_prefix("hw.optional.arm.")
                        .expect("starts_with guard ensures this matches");
                    feat.to_lowercase()
                } else if key.starts_with("hw.optional.") {
                    let feat = key
                        .strip_prefix("hw.optional.")
                        .expect("starts_with guard ensures this matches");
                    feat.to_lowercase()
                } else {
                    continue;
                };

                // Map known feature names to canonical lowercase names
                let canonical = match feature_name.as_str() {
                    "floatingpoint" => "fp",
                    "neon" => "neon",
                    "neon_hpfp" => "fphp",
                    "neon_fp16" => "fp16",
                    "armv8_1_atomics" => "atomics",
                    "armv8_crc32" => "crc32",
                    "armv8_2_fhm" => "asimdfhm",
                    "armv8_2_sha512" => "sha512",
                    "armv8_2_sha3" => "sha3",
                    "amx_version" => "amx",
                    "ucnormal_mem" => "ucnormal",
                    "arm64" => "asimd", // arm64 implies ASIMD
                    // hw.optional.arm.FEAT_* names
                    "crc32" => "crc32",
                    "flagm" => "flagm",
                    "fhm" => "asimdfhm",
                    "dotprod" => "dotprod",
                    "sha3" => "sha3",
                    "rdm" => "asimdrdm",
                    "lse" => "atomics",
                    "sha256" => "sha2",
                    "sha512" => "sha512",
                    "sha1" => "sha1",
                    "aes" => "aes",
                    "pmull" => "pmull",
                    "specres" => "specres",
                    "specres2" => "specres2",
                    "sb" => "sb",
                    "frintts" => "frintts",
                    "lrcpc" => "lrcpc",
                    "lrcpc2" => "lrcpc2",
                    "fcma" => "fcma",
                    "jscvt" => "jscvt",
                    "pauth" => "pauth",
                    "pauth2" => "pauth2",
                    "fpac" => "fpac",
                    "fpaccomb" => "fpac", // alias
                    "dpb" => "dpb",
                    "dpb2" => "dpb2",
                    "bf16" => "bf16",
                    "ebf16" => "bf16", // alias
                    "i8mm" => "i8mm",
                    "wft" => "wfx",
                    "rpres" => "rpres",
                    "cssc" => "cssc",
                    "hbc" => "hbc",
                    "ecv" => "ecv",
                    "afp" => "afp",
                    "lse2" => "lse2",
                    "csv2" => "csv2",
                    "csv3" => "csv3",
                    "pacimp" => "pauth",
                    _ => &feature_name,
                };

                features.insert(canonical.to_string(), true);
            }
        }
    }

    // Default features for Apple Silicon (always present)
    if features.is_empty() {
        // If sysctl didn't work, assume M1+ baseline features
        features.insert("fp".to_string(), true);
        features.insert("asimd".to_string(), true);
        features.insert("neon".to_string(), true);
        features.insert("evtstrm".to_string(), true);
        features.insert("aes".to_string(), true);
        features.insert("pmull".to_string(), true);
        features.insert("sha1".to_string(), true);
        features.insert("sha2".to_string(), true);
        features.insert("crc32".to_string(), true);
        features.insert("atomics".to_string(), true);
        features.insert("asimdrdm".to_string(), true);
        features.insert("jscvt".to_string(), true);
        features.insert("fcma".to_string(), true);
        features.insert("lrcpc".to_string(), true);
        features.insert("dcpop".to_string(), true);
        features.insert("asimddp".to_string(), true);
        features.insert("sve".to_string(), true);
    }

    features
}

impl TArmFeatures for crate::arm::ArmFeatures {
    fn has_fp(&self) -> bool {
        let features = get_features_from_sysctl();
        features.get("fp").copied().unwrap_or(false)
    }

    fn has_asimd(&self) -> bool {
        let features = get_features_from_sysctl();
        features.get("asimd").copied().unwrap_or(false)
            || features.get("neon").copied().unwrap_or(false)
    }

    fn has_aes(&self) -> bool {
        let features = get_features_from_sysctl();
        features.get("aes").copied().unwrap_or(false)
    }

    fn has_sha1(&self) -> bool {
        let features = get_features_from_sysctl();
        features.get("sha1").copied().unwrap_or(false)
    }

    fn has_sha2(&self) -> bool {
        let features = get_features_from_sysctl();
        features.get("sha2").copied().unwrap_or(false)
    }

    fn has_sha3(&self) -> bool {
        let features = get_features_from_sysctl();
        features.get("sha3").copied().unwrap_or(false)
    }

    fn has_sha512(&self) -> bool {
        let features = get_features_from_sysctl();
        features.get("sha512").copied().unwrap_or(false)
    }

    fn has_crc32(&self) -> bool {
        let features = get_features_from_sysctl();
        features.get("crc32").copied().unwrap_or(false)
    }

    fn has_atomics(&self) -> bool {
        let features = get_features_from_sysctl();
        features.get("atomics").copied().unwrap_or(false)
    }
}

/// Returns all detected features as a BTreeMap of category to space-separated features.
pub fn get_all_features() -> BTreeMap<&'static str, String> {
    let src = get_features_from_sysctl();
    let detected = crate::arm::features::populate_detected_features(&src);
    crate::arm::features::build_feature_map(&detected)
}

const CPUFAMILY_ARM_FIRESTORM_ICESTORM: usize = 0x1b588bb3;
const CPUFAMILY_ARM_BLIZZARD_AVALANCHE: usize = 0xda33d83d;
const CPUFAMILY_ARM_EVEREST_SAWTOOTH: usize = 0x8765edea;

/// Get all the juicy cpu details from sysctl
pub(crate) fn get_sysctl_map() -> BTreeMap<String, String> {
    let mut values: BTreeMap<String, String> = BTreeMap::new();

    for (key, value) in get_full_raw_sysctl_map() {
        if key.starts_with("machdep.cpu") || (key.starts_with("hw") && !key.contains("optional")) {
            values.insert(key.clone(), value.clone());
        }
    }

    values
}

pub fn get_synth_midr() -> usize {
    let values = get_sysctl_map();

    let cpufamily = if let Some(family) = values.get("hw.cpufamily") {
        family.parse::<usize>().ok()
    } else {
        None
    };

    let brand_string = values.get("machdep.cpu.brand_string");

    if let (Some(family), Some(brand)) = (cpufamily, brand_string) {
        cpufamily_to_midr(family, brand)
    } else {
        0
    }
}

fn cpufamily_to_midr(cpufamily: usize, brand_string: &str) -> usize {
    let midr_base = IMPL_APPLE << IMPLEMENTER_OFFSET;

    match cpufamily {
        // M1 family
        CPUFAMILY_ARM_FIRESTORM_ICESTORM => {
            if brand_string.contains("M1 Pro") {
                midr_base | (0x024 << PART_OFFSET)
            } else if brand_string.contains("M1 Max") {
                midr_base | (0x028 << PART_OFFSET)
            } else {
                midr_base | (0x022 << PART_OFFSET) // M1 base
            }
        }

        // M2 Family
        CPUFAMILY_ARM_BLIZZARD_AVALANCHE => {
            if brand_string.contains("M2 Pro") {
                midr_base | (0x034 << PART_OFFSET)
            } else if brand_string.contains("M2 Max") {
                midr_base | (0x038 << PART_OFFSET)
            } else {
                midr_base | (0x030 << PART_OFFSET) // A15, M2 base
            }
        }

        // M3 family
        CPUFAMILY_ARM_EVEREST_SAWTOOTH => {
            if brand_string.contains("M3 Pro") {
                midr_base | (0x044 << PART_OFFSET)
            } else if brand_string.contains("M3 Max") {
                midr_base | (0x048 << PART_OFFSET)
            } else {
                midr_base | (0x042 << PART_OFFSET) // A16, M3 base
            }
        }

        // M4 family
        0x4B4FAE0A => {
            if brand_string.contains("M4 Pro") {
                midr_base | (0x054 << PART_OFFSET)
            } else if brand_string.contains("M4 Max") {
                midr_base | (0x058 << PART_OFFSET)
            } else {
                midr_base | (0x052 << PART_OFFSET) // M4 base
            }
        }

        // Apple A18 / A18 Pro (0x75D4ACB9)
        0x75D4ACB9 => {
            if brand_string.contains("A18 Pro") {
                midr_base | (0x101 << PART_OFFSET)
            } else {
                midr_base | (0x100 << PART_OFFSET) // A18
            }
        }

        _ => 0,
    }
}

/// Maps an Apple MIDR part number and core type to a MicroArch.
/// This is macOS-specific because sysctl provides perflevels (core types)
/// while the synthesized MIDR only represents one part number per family.
fn find_core_micro_arch(midr: &Midr, kind: CoreType) -> MicroArch {
    match (midr.part, kind) {
        // M1
        (0x022..=0x029, CoreType::Performance) => MicroArch::AppleFirestorm,
        (0x022..=0x029, CoreType::Efficiency) => MicroArch::AppleIcestorm,

        // M2
        (0x030..=0x039, CoreType::Performance) => MicroArch::AppleAvalanche,
        (0x030..=0x039, CoreType::Efficiency) => MicroArch::AppleBlizzard,

        // M3
        (0x040..=0x049, CoreType::Performance) => MicroArch::AppleEverest,
        (0x040..=0x049, CoreType::Efficiency) => MicroArch::AppleSawtooth,

        // M4
        (0x050..=0x059, CoreType::Performance) => MicroArch::AppleEverest,
        (0x050..=0x059, CoreType::Efficiency) => MicroArch::AppleSawtooth,

        // A18 Pro
        (0x101, CoreType::Performance) => MicroArch::AppleEverest,
        (0x101, CoreType::Efficiency) => MicroArch::AppleSawtooth,

        // A16
        (0x036..=0x037, CoreType::Performance) => MicroArch::AppleEverest,
        (0x036..=0x037, CoreType::Efficiency) => MicroArch::AppleSawtooth,

        // A15
        (0x030..=0x031, CoreType::Performance) => MicroArch::AppleAvalanche,
        (0x030..=0x031, CoreType::Efficiency) => MicroArch::AppleBlizzard,

        // A14
        (0x020..=0x021, CoreType::Performance) => MicroArch::AppleFirestorm,
        (0x020..=0x021, CoreType::Efficiency) => MicroArch::AppleIcestorm,

        // A13
        (0x012..=0x013, CoreType::Performance) => MicroArch::AppleLightning,
        (0x012..=0x013, CoreType::Efficiency) => MicroArch::AppleThunder,

        // A12
        (0x00B..=0x00C, CoreType::Performance) => MicroArch::AppleVortex,
        (0x00B..=0x00C, CoreType::Efficiency) => MicroArch::AppleTempest,

        // A11
        (0x008..=0x009, CoreType::Performance) => MicroArch::AppleMonsoon,
        (0x008..=0x009, CoreType::Efficiency) => MicroArch::AppleMistral,

        // A10
        (0x006, _) => MicroArch::AppleHurricane,

        // A9
        (0x004, _) => MicroArch::AppleTwister,
        // A8
        (0x002, _) => MicroArch::AppleTyphoon,
        // A7
        (0x001, _) => MicroArch::AppleCyclone,
        // A6
        (0x000, _) => MicroArch::AppleSwift,

        _ => MicroArch::Unknown,
    }
}

/// macOS-specific CPU detection via sysctl.
pub fn detect() -> OsCpuInfo {
    let midr_val = get_synth_midr();
    let midr = Midr::new(midr_val);
    let mut midrs = HashSet::new();
    midrs.insert(midr);

    let vendor: String = Vendor::from(midr.implementer).into();
    let cpu_arch = CpuArch::find(midr.implementer, midr.part, midr.variant);
    let values = get_sysctl_map();
    let mut cores: BTreeMap<(CoreType, Midr), CpuCore> = BTreeMap::new();

    let perf_levels: usize = values
        .get("hw.nperflevels")
        .expect("sysctl hw.nperflevels missing")
        .parse()
        .expect("sysctl hw.nperflevels not a valid usize");

    for i in 0..perf_levels {
        let kind_type = values.get(&format!("hw.perflevel{}.name", i));
        let kind = CoreType::from(kind_type.expect("sysctl perflevel name missing").clone());
        let mut cache = Cache::default();
        let mut l1 = Level1Cache::default_split();

        let cpus_per_l2: u32 = values
            .get(&format!("hw.perflevel{}.cpusperl2", i))
            .expect("sysctl perflevel cpusperl2 missing")
            .parse()
            .expect("sysctl perflevel cpusperl2 not a valid u32");
        let l1d_size: u32 = values
            .get(&format!("hw.perflevel{}.l1dcachesize", i))
            .expect("sysctl perflevel l1dcachesize missing")
            .parse()
            .expect("sysctl perflevel l1dcachesize not a valid u32");
        let l1i_size: u32 = values
            .get(&format!("hw.perflevel{}.l1icachesize", i))
            .expect("sysctl perflevel l1icachesize missing")
            .parse()
            .expect("sysctl perflevel l1icachesize not a valid u32");
        let l2_size: u32 = values
            .get(&format!("hw.perflevel{}.l2cachesize", i))
            .expect("sysctl perflevel l2cachesize missing")
            .parse()
            .expect("sysctl perflevel l2cachesize not a valid u32");
        let count: u32 = values
            .get(&format!("hw.perflevel{}.physicalcpu", i))
            .expect("sysctl perflevel physicalcpu missing")
            .parse()
            .expect("sysctl perflevel physicalcpu not a valid u32");

        l1.set_data(l1d_size, 0);
        l1.set_data_share_count(1);
        l1.set_instruction(l1i_size, 0);
        l1.set_instruction_share_count(1);
        cache.l1 = l1;
        cache.l2 = Some(CacheLevel::new(l2_size, CacheType::Unified, 0, cpus_per_l2));

        let micro_arch = find_core_micro_arch(&midr, kind);

        cores.insert(
            (kind, midr),
            CpuCore {
                implementer: Vendor::Apple,
                kind,
                micro_arch,
                code_name: None,
                cache: Some(cache),
                count,
            },
        );
    }

    let model = values
        .get("machdep.cpu.brand_string")
        .expect("sysctl machdep.cpu.brand_string missing")
        .to_string();

    OsCpuInfo {
        midrs,
        vendor,
        cpu_arch,
        cores,
        model,
        raw: values,
        midr_source: DataSource::Sysctrl("hw.cpufamily"),
        features_source: DataSource::Sysctrl("hw.optional.*"),
    }
}
