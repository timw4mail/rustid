use super::CpuDisplay;
use super::brand::*;
use super::micro_arch::CpuArch;
use super::micro_arch::*;
use crate::arm::TArmCpu;
use crate::common::UNK;
use crate::common::*;
use std::collections::{BTreeMap, HashSet};
use std::process::Command;

// ----------------------------------------------------------------------------
// Feature detection via sysctl (text-based, matches existing pattern)
// ----------------------------------------------------------------------------

/// Parses sysctl output for hw.optional.* keys to detect CPU features.
/// All feature names are converted to lowercase for consistency.
pub fn get_features_from_sysctl() -> BTreeMap<String, bool> {
    let mut features: BTreeMap<String, bool> = BTreeMap::new();

    // Run sysctl to get hw.optional and hw.optional.arm keys
    if let Ok(output) = Command::new("sysctl").arg("-a").output()
        && let Ok(stdout) = String::from_utf8(output.stdout)
    {
        for line in stdout.lines() {
            let line = line.trim();
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();

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
                            let feat = key.strip_prefix("hw.optional.arm.FEAT_").unwrap();
                            feat.to_lowercase()
                        } else if key.starts_with("hw.optional.arm.FEAT") {
                            // Handle "hw.optional.arm.FEATXYZ" without underscore
                            let feat = key.strip_prefix("hw.optional.arm.FEAT").unwrap();
                            feat.to_lowercase()
                        } else if key.starts_with("hw.optional.arm.") {
                            let feat = key.strip_prefix("hw.optional.arm.").unwrap();
                            feat.to_lowercase()
                        } else if key.starts_with("hw.optional.") {
                            let feat = key.strip_prefix("hw.optional.").unwrap();
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

/// Check if floating-point (fp) is supported.
pub fn has_fp() -> bool {
    let features = get_features_from_sysctl();
    features.get("fp").copied().unwrap_or(false)
}

/// Check if Advanced SIMD (NEON/asimd) is supported.
pub fn has_simd() -> bool {
    let features = get_features_from_sysctl();
    features.get("asimd").copied().unwrap_or(false)
        || features.get("neon").copied().unwrap_or(false)
}

/// Check if NEON is supported (alias for has_simd on ARM).
pub fn has_neon() -> bool {
    has_simd()
}

/// Check if AES instructions are supported.
pub fn has_aes() -> bool {
    let features = get_features_from_sysctl();
    features.get("aes").copied().unwrap_or(false)
}

/// Check if SHA1 instructions are supported.
pub fn has_sha1() -> bool {
    let features = get_features_from_sysctl();
    features.get("sha1").copied().unwrap_or(false)
}

/// Check if SHA2 instructions are supported.
pub fn has_sha2() -> bool {
    let features = get_features_from_sysctl();
    features.get("sha2").copied().unwrap_or(false)
}

/// Check if SHA3 instructions are supported.
pub fn has_sha3() -> bool {
    let features = get_features_from_sysctl();
    features.get("sha3").copied().unwrap_or(false)
}

/// Check if SHA512 instructions are supported.
pub fn has_sha512() -> bool {
    let features = get_features_from_sysctl();
    features.get("sha512").copied().unwrap_or(false)
}

/// Check if CRC32 instructions are supported.
pub fn has_crc32() -> bool {
    let features = get_features_from_sysctl();
    features.get("crc32").copied().unwrap_or(false)
}

/// Check if atomic instructions (LSE) are supported.
pub fn has_atomics() -> bool {
    let features = get_features_from_sysctl();
    features.get("atomics").copied().unwrap_or(false)
}

/// Returns all detected features as a BTreeMap of category to space-separated features.
pub fn get_all_features() -> BTreeMap<&'static str, String> {
    let mut detected: BTreeMap<&'static str, bool> = BTreeMap::new();
    let features = get_features_from_sysctl();

    // Base features
    detected.insert("fp", features.get("fp").copied().unwrap_or(false));
    detected.insert("asimd", features.get("asimd").copied().unwrap_or(false));
    detected.insert("cpuid", features.get("cpuid").copied().unwrap_or(false));
    detected.insert("evtstrm", features.get("evtstrm").copied().unwrap_or(false));

    // SIMD
    detected.insert("neon", features.get("neon").copied().unwrap_or(false));
    detected.insert("asimdhp", features.get("asimdhp").copied().unwrap_or(false));
    detected.insert(
        "asimdfhm",
        features.get("asimdfhm").copied().unwrap_or(false),
    );
    detected.insert("asimddp", features.get("asimddp").copied().unwrap_or(false));
    detected.insert(
        "asimdrdm",
        features.get("asimdrdm").copied().unwrap_or(false),
    );

    // Crypto
    detected.insert("aes", features.get("aes").copied().unwrap_or(false));
    detected.insert("pmull", features.get("pmull").copied().unwrap_or(false));
    detected.insert("sha1", features.get("sha1").copied().unwrap_or(false));
    detected.insert("sha2", features.get("sha2").copied().unwrap_or(false));
    detected.insert("sha3", features.get("sha3").copied().unwrap_or(false));
    detected.insert("sha512", features.get("sha512").copied().unwrap_or(false));
    detected.insert("sm3", features.get("sm3").copied().unwrap_or(false));
    detected.insert("sm4", features.get("sm4").copied().unwrap_or(false));

    // Atomic
    detected.insert("atomics", features.get("atomics").copied().unwrap_or(false));
    detected.insert("lse", features.get("atomics").copied().unwrap_or(false));
    detected.insert("lse2", features.get("lse2").copied().unwrap_or(false));

    // FP
    detected.insert("fphp", features.get("fphp").copied().unwrap_or(false));
    detected.insert("fp16", features.get("fp16").copied().unwrap_or(false));
    detected.insert("fcma", features.get("fcma").copied().unwrap_or(false));
    detected.insert("jscvt", features.get("jscvt").copied().unwrap_or(false));

    // Misc
    detected.insert("crc32", features.get("crc32").copied().unwrap_or(false));
    detected.insert("dcpop", features.get("dcpop").copied().unwrap_or(false));
    detected.insert("lrcpc", features.get("lrcpc").copied().unwrap_or(false));
    detected.insert("lrcpc2", features.get("lrcpc2").copied().unwrap_or(false));
    detected.insert("flagm", features.get("flagm").copied().unwrap_or(false));
    detected.insert("flagm2", features.get("flagm2").copied().unwrap_or(false));
    detected.insert("dit", features.get("dit").copied().unwrap_or(false));
    detected.insert("ssbs", features.get("ssbs").copied().unwrap_or(false));
    detected.insert("bti", features.get("bti").copied().unwrap_or(false));
    detected.insert("pauth", features.get("pauth").copied().unwrap_or(false));
    detected.insert("pauth2", features.get("pauth2").copied().unwrap_or(false));
    detected.insert("fpac", features.get("fpac").copied().unwrap_or(false));
    detected.insert("specres", features.get("specres").copied().unwrap_or(false));
    detected.insert(
        "specres2",
        features.get("specres2").copied().unwrap_or(false),
    );
    detected.insert("csv2", features.get("csv2").copied().unwrap_or(false));
    detected.insert("csv3", features.get("csv3").copied().unwrap_or(false));
    detected.insert("ecv", features.get("ecv").copied().unwrap_or(false));
    detected.insert("sb", features.get("sb").copied().unwrap_or(false));
    detected.insert("frintts", features.get("frintts").copied().unwrap_or(false));
    detected.insert("dpb", features.get("dpb").copied().unwrap_or(false));
    detected.insert("dpb2", features.get("dpb2").copied().unwrap_or(false));
    detected.insert("dotprod", features.get("dotprod").copied().unwrap_or(false));
    detected.insert("bf16", features.get("bf16").copied().unwrap_or(false));
    detected.insert("i8mm", features.get("i8mm").copied().unwrap_or(false));
    detected.insert("sve", features.get("sve").copied().unwrap_or(false));
    detected.insert("sve2", features.get("sve2").copied().unwrap_or(false));
    detected.insert("sme", features.get("sme").copied().unwrap_or(false));

    super::features::build_feature_map(&detected)
}

const CPUFAMILY_ARM_FIRESTORM_ICESTORM: usize = 0x1b588bb3;
const CPUFAMILY_ARM_BLIZZARD_AVALANCHE: usize = 0xda33d83d;
const CPUFAMILY_ARM_EVEREST_SAWTOOTH: usize = 0x8765edea;

/// Get all the juicy cpu details from sysctl
fn get_sysctl_map() -> BTreeMap<String, String> {
    let mut values: BTreeMap<String, String> = BTreeMap::new();
    TryInto::<String>::try_into(
        Command::new("sysctl")
            .arg("-a")
            .output()
            .expect("Failed to load cpu details from sysctl")
            .stdout,
    )
    .unwrap()
    .split('\n')
    .filter(|l| !l.is_empty())
    .for_each(|x| {
        let line: Vec<_> = x.split(": ").collect();
        if let Some(key) = line.first()
            && let Some(val) = line.get(1)
            && (key.starts_with("machdep.cpu")
                || (key.starts_with("hw") && !key.contains("optional")))
        {
            values.insert(String::from(*key), String::from(*val));
        }
    });

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
    let midr_base = IMPL_APPLE << 24;

    match cpufamily {
        // M1 family
        CPUFAMILY_ARM_FIRESTORM_ICESTORM => {
            if brand_string.contains("M1 Pro") {
                midr_base | (0x024 << 4)
            } else if brand_string.contains("M1 Max") {
                midr_base | (0x028 << 4)
            } else {
                midr_base | (0x022 << 4) // M1 base
            }
        }

        // M2 Family
        CPUFAMILY_ARM_BLIZZARD_AVALANCHE => {
            if brand_string.contains("M2 Pro") {
                midr_base | (0x034 << 4)
            } else if brand_string.contains("M2 Max") {
                midr_base | (0x038 << 4)
            } else {
                midr_base | (0x030 << 4) // A15, M2 base
            }
        }

        // M3 family
        CPUFAMILY_ARM_EVEREST_SAWTOOTH => {
            if brand_string.contains("M3 Pro") {
                midr_base | (0x044 << 4)
            } else if brand_string.contains("M3 Max") {
                midr_base | (0x048 << 4)
            } else {
                midr_base | (0x042 << 4) // A16, M3 base
            }
        }

        // M4 family
        0x4B4FAE0A => {
            if brand_string.contains("M4 Pro") {
                midr_base | (0x054 << 4)
            } else if brand_string.contains("M4 Max") {
                midr_base | (0x058 << 4)
            } else {
                midr_base | (0x052 << 4) // M4 base
            }
        }

        // Apple A18 / A18 Pro (0x75D4ACB9)
        0x75D4ACB9 => {
            if brand_string.contains("A18 Pro") {
                midr_base | (0x101 << 4)
            } else {
                midr_base | (0x100 << 4) // A18
            }
        }

        _ => 0,
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Cpu {
    pub raw_midr: HashSet<usize>,
    pub midrs: HashSet<Midr>,
    pub vendor: String,
    pub cpu_arch: CpuArch,
    pub model: String,
    pub cores: BTreeMap<(CoreType, Option<String>, Midr), CpuCore>,
    pub raw: BTreeMap<String, String>,
    pub features: BTreeMap<&'static str, String>,
}

impl TCpu for Cpu {
    fn detect() -> Self {
        let mut raw_midr: HashSet<usize> = HashSet::new();
        let mut midrs: HashSet<Midr> = HashSet::new();

        let midr_val = get_synth_midr();
        raw_midr.insert(midr_val);
        let midr = Midr::new(midr_val);
        midrs.insert(midr);

        let vendor = Vendor::from(midr.implementer);
        let cpu_arch = CpuArch::find(midr.implementer, midr.part, midr.variant);
        let values = get_sysctl_map();
        let mut cores: BTreeMap<(CoreType, Option<String>, Midr), CpuCore> = BTreeMap::new();

        let perf_levels: usize = values.get("hw.nperflevels").unwrap().parse().unwrap();

        for i in 0..perf_levels {
            let kind_type = values.get(&format!("hw.perflevel{}.name", i));
            let kind = CoreType::from(kind_type.unwrap().clone());
            let mut cache = Cache::default();
            let mut l1 = Level1Cache::default_split();

            let cpus_per_l2: u32 = values
                .get(&format!("hw.perflevel{}.cpusperl2", i))
                .unwrap()
                .parse()
                .unwrap();
            let l1d_size: u32 = values
                .get(&format!("hw.perflevel{}.l1dcachesize", i))
                .unwrap()
                .parse()
                .unwrap();
            let l1i_size: u32 = values
                .get(&format!("hw.perflevel{}.l1icachesize", i))
                .unwrap()
                .parse()
                .unwrap();
            let l2_size: u32 = values
                .get(&format!("hw.perflevel{}.l2cachesize", i))
                .unwrap()
                .parse()
                .unwrap();
            let count: u32 = values
                .get(&format!("hw.perflevel{}.physicalcpu", i))
                .unwrap()
                .parse()
                .unwrap();

            l1.set_data(l1d_size, 0);
            l1.set_data_share_count(1);
            l1.set_instruction(l1i_size, 0);
            l1.set_instruction_share_count(1);
            cache.l1 = l1;
            cache.l2 = Some(CacheLevel::new(l2_size, CacheType::Unified, 0, cpus_per_l2));

            let name = Self::find_core_codename(&midr, kind);

            cores.insert(
                (kind, name.clone(), midr),
                CpuCore {
                    kind,
                    name,
                    cache: Some(cache),
                    count,
                },
            );
        }

        let features = super::get_all_features();

        Self {
            model: values.get("machdep.cpu.brand_string").unwrap().to_string(),
            raw_midr,
            midrs,
            vendor: vendor.into(),
            cpu_arch,
            cores,
            raw: values,
            features,
        }
    }

    fn debug(&self)
    where
        Self: std::fmt::Debug,
    {
        println!(
            "Main ID Register (MIDR): 0x{:X}",
            self.raw_midr().iter().next().unwrap_or(&0)
        );
        if let Some(midr) = self.midr() {
            println!("Implementer: 0x{:X} ({})", midr.implementer, self.vendor());
            println!("Variant: 0x{:X}", midr.variant);
            println!("Part Number: 0x{:X}", midr.part);
            println!("Revision: 0x{:X}", midr.revision);
        }
        println!("{:#?}", self);
    }

    fn display_table(&self, color: bool) {
        CpuDisplay::display(&self.cpu_arch, &self.cores, &self.features, color);
    }
}

impl TArmCpu for Cpu {
    fn model(&self) -> Option<&str> {
        Some(&self.model)
    }

    fn raw_midr(&self) -> HashSet<usize> {
        self.raw_midr.clone()
    }

    fn midr(&self) -> Option<&Midr> {
        self.midrs.iter().next()
    }

    fn vendor(&self) -> &str {
        &self.vendor
    }
}

impl Cpu {
    fn find_core_codename(midr: &Midr, kind: CoreType) -> Option<String> {
        let str = match (midr.part, kind) {
            // M1
            (0x022..=0x029, CoreType::Performance) => "FireStorm",
            (0x022..=0x029, CoreType::Efficiency) => "IceStorm",

            // M2
            (0x030..=0x039, CoreType::Performance) => "Avalanche",
            (0x030..=0x039, CoreType::Efficiency) => "Blizzard",

            // M3+, A18 Pro
            (0x101 | 0x040..=0x059, CoreType::Performance) => "Everest",
            (0x101 | 0x040..=0x059, CoreType::Efficiency) => "Sawtooth",

            _ => UNK,
        };

        if str == UNK {
            None
        } else {
            Some(String::from(str))
        }
    }
}
