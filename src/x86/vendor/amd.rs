//! AMD-specific CPU microarchitecture detection and signature mapping.
//!
//! # References and Sources
//! - AMD Processor Programming References (PPR):
//!   - Family 17h Models 00h-0Fh (Pub #54945)
//!   - Family 17h Model 18h (Pub #55570)
//!   - Family 17h Model 20h (Pub #55772)
//!   - Family 17h Models 30h-3Fh (Pub #55803)
//!   - Family 17h Model 60h (Pub #55922)
//!   - Family 17h Model 70h (Pub #56323)
//!   - Family 19h Model 01h (Pub #55898)
//!   - Family 19h Models 10h-1Fh (Pub #55901)
//!   - Family 19h Model 21h (Pub #56569)
//!   - Family 19h Model 51h (Pub #56569)
//!   - Family 19h Model 70h/74h (Pub #57019)
//!   - Family 19h Models A0h-AFh (Pub #57228)
//!   - Family 1Ah Models 00h-0Fh (Pub #57238)
//!   - Family 1Ah Models 20h-2Fh / 40h-4Fh (Pub #57243)
//! - AMD BIOS and Kernel Developer's Guides (BKDG):
//!   - Family 10h BKDG (Pub #31116)
//!   - Family 11h BKDG (Pub #41256)
//!   - Family 12h BKDG (Pub #41131)
//!   - Family 14h BKDG (Pub #43009)
//!   - Family 15h Models 00h-0Fh / 10h-1Fh / 30h-3Fh / 60h-6Fh / 70h-7Fh (Pubs #42301, #42300, #49125, #50742, #55072)
//!   - Family 16h Models 00h-0Fh / 30h-3Fh (Pubs #48751, #52740)
//! - Linux kernel `arch/x86/kernel/cpu/amd.c`, `arch/x86/events/amd/core.c`, `drivers/edac/amd64_edac.c`
//! - InstLatx64 CPUID dumps & WikiChip AMD CPUID tables.

use crate::x86::CpuSignature;
#[cfg(not(dos))]
use crate::x86::amd_logical_cores;
use crate::x86::constants::*;
use crate::x86::micro_arch::{CpuArch, MicroArch};
use crate::x86::vendor::TMicroArch;

/// AMD-specific microarchitecture detection.
pub struct Amd;

impl Amd {
    /// Detects legacy AMD CPUs (486, 5x86) supported in all environments including DOS.
    fn legacy_micro_arch(
        s: CpuSignature,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> Option<CpuArch> {
        let arch = match (
            s.extended_family,
            s.family,
            s.extended_model,
            s.model,
            s.stepping,
        ) {
            // Am486 & Am5x86
            (0, 4, 0, 0, _) => brand_arch(MicroArch::Am486, "Am486DX", None),
            (0, 4, 0, 1, _) => brand_arch(MicroArch::Am486, "Am486DX-40", None),
            (0, 4, 0, 2, _) => brand_arch(MicroArch::Am486, "Am486SX", None),
            (0, 4, 0, 3, _) => brand_arch(MicroArch::Am486, "Am486DX2", None),
            (0, 4, 0, 7, _) => brand_arch(MicroArch::Am486, "Am486X2WB", None),
            (0, 4, 0, 8, _) => brand_arch(MicroArch::Am486, "Am486DX4", None),
            (0, 4, 0, 9, _) => brand_arch(MicroArch::Am486, "Am486DX4WB", None),
            (0, 4, 0, 14, _) => brand_arch(MicroArch::Am5x86, "Am5x86", None),
            (0, 4, 0, 15, _) => brand_arch(MicroArch::Am5x86, "Am5x86WB", None),
            _ => return None,
        };
        Some(arch)
    }

    #[cfg(not(dos))]
    fn modern_micro_arch(
        model: &str,
        s: CpuSignature,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> Option<CpuArch> {
        let m_lower = model.to_ascii_lowercase();

        let arch = match (
            s.extended_family,
            s.family,
            s.extended_model,
            s.model,
            s.stepping,
        ) {
            // ================================================================
            // Family 5: K5, K6, Geode LX
            // ================================================================
            (0, 5, 0, 0, _) => brand_arch(MicroArch::SSA5, "SSA/5", Some(N350)),
            (0, 5, 0, 1..=3, _) => brand_arch(MicroArch::K5, "5k86", Some(N350)),
            (0, 5, 0, 6, _) => brand_arch(MicroArch::K6, "Model 6", Some(N300)),
            (0, 5, 0, 7, _) => brand_arch(MicroArch::K6, "Little Foot", Some(N250)),
            (0, 5, 0, 8, _) => brand_arch(MicroArch::K6, "Chompers/CXT", Some(N250)), // K6-2
            (0, 5, 0, 9, _) => brand_arch(MicroArch::K6, "Sharptooth", Some(N250)),   // K6-III
            (0, 5, 0, 10, _) => brand_arch(MicroArch::Geode, "Geode LX", Some(N130)),
            (0, 5, 0, 12 | 13, _) => brand_arch(MicroArch::K6, "Sharptooth", Some(N180)), // K6-2+ / K6-III+

            // ================================================================
            // Family 6: K7 (Athlon, Duron, Sempron, Geode NX)
            // ================================================================
            (0, 6, 0, 1, _) => brand_arch(MicroArch::K7, "Argon", Some(N250)),
            (0, 6, 0, 2, _) => brand_arch(MicroArch::K7, "Pluto/Orion", Some(N180)),
            (0, 6, 0, 3, _) => brand_arch(MicroArch::K7, "Spitfire", Some(N180)),
            (0, 6, 0, 4, _) => brand_arch(MicroArch::K7, "Thunderbird", Some(N180)),
            (0, 6, 0, 6, _) => brand_arch(MicroArch::K7, "Palomino", Some(N180)),
            (0, 6, 0, 7, _) => brand_arch(MicroArch::K7, "Morgan", Some(N180)),
            (0, 6, 0, 8, _) => {
                if m_lower.contains("applebred") {
                    brand_arch(MicroArch::K7, "Applebred", Some(N130))
                } else if m_lower.contains("geode") {
                    brand_arch(MicroArch::K7, "Geode NX", Some(N130))
                } else if m_lower.contains("sempron") {
                    brand_arch(MicroArch::K7, "Thoroughbred (Sempron)", Some(N130))
                } else {
                    brand_arch(MicroArch::K7, "Thoroughbred", Some(N130))
                }
            }
            (0, 6, 0, 10, _) => {
                if m_lower.contains("thorton") {
                    brand_arch(MicroArch::K7, "Thorton", Some(N130))
                } else {
                    brand_arch(MicroArch::K7, "Barton", Some(N130))
                }
            }

            // ================================================================
            // Family 0Fh: K8 (Hammer / Opteron / Athlon 64 / X2 / Turion 64)
            // ================================================================
            (0, 15, _, _, _) => Self::disambiguate_k8(&m_lower, s, brand_arch),

            // ================================================================
            // Family 10h: K10 (Stars / Phenom / Phenom II / Athlon II / Opteron)
            // ================================================================
            (1, 15, _, _, _) => Self::disambiguate_k10(&m_lower, s, brand_arch),

            // ================================================================
            // Family 11h: Griffin / Turion X2 Ultra (Puma 2008)
            // ================================================================
            (2, 15, 0, 0..=3, _) => brand_arch(MicroArch::Puma2008, "Griffin", Some(N65)),

            // ================================================================
            // Family 12h: Llano / Husky (Stars K10.5 APU)
            // ================================================================
            (3, 15, 0, 0..=3, _) => brand_arch(MicroArch::K10, "Llano", Some(N32)),

            // ================================================================
            // Family 14h: Bobcat APU (Ontario / Zacate / Desna / Hondo)
            // ================================================================
            (5, 15, 0, 1, _) => brand_arch(MicroArch::Bobcat, "Ontario", Some(N40)),
            (5, 15, 0, 2, _) => brand_arch(MicroArch::Bobcat, "Zacate", Some(N40)),
            (5, 15, 0, 0 | 3, _) => brand_arch(MicroArch::Bobcat, "Desna/Hondo", Some(N40)),

            // ================================================================
            // Family 15h: Bulldozer / Piledriver / Steamroller / Excavator
            // ================================================================
            (6, 15, _, _, _) => Self::disambiguate_fam15h(&m_lower, s, brand_arch),

            // ================================================================
            // Family 16h: Jaguar / Puma 2014
            // ================================================================
            (7, 15, _, _, _) => Self::disambiguate_fam16h(&m_lower, s, brand_arch),

            // ================================================================
            // Family 17h: Zen 1, Zen+, Zen 2
            // ================================================================
            (8, 15, _, _, _) => Self::disambiguate_fam17h(&m_lower, s, brand_arch),

            // ================================================================
            // Family 19h: Zen 3, Zen 3+, Zen 4, Zen 4c
            // ================================================================
            (10, 15, _, _, _) => Self::disambiguate_fam19h(&m_lower, s, brand_arch),

            // ================================================================
            // Family 1Ah: Zen 5, Zen 5c
            // ================================================================
            (11, 15, _, _, _) => Self::disambiguate_fam1ah(&m_lower, s, brand_arch),

            // ================================================================
            // Family 1Bh: Zen 6, Zen 6c (Future)
            // ================================================================
            (12, 15, _, _, _) => brand_arch(MicroArch::Zen6, "Morpheus", Some(N3)),

            _ => return None,
        };

        Some(arch)
    }

    #[cfg(not(dos))]
    fn disambiguate_k8(
        m_lower: &str,
        s: CpuSignature,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> CpuArch {
        let is_dual_core = amd_logical_cores() > 1;

        match (s.extended_model, s.model) {
            // 130nm ClawHammer / SledgeHammer / Newcastle / Paris / Dublin
            (0, 4) => brand_arch(MicroArch::K8, "ClawHammer", Some(N130)),
            (0, 5) => {
                if m_lower.contains("opteron") {
                    brand_arch(MicroArch::K8, "SledgeHammer", Some(N130))
                } else {
                    brand_arch(MicroArch::K8, "ClawHammer", Some(N130))
                }
            }
            (0, 7 | 8) => brand_arch(MicroArch::K8, "ClawHammer", Some(N130)),
            (0, 11 | 14) => {
                if m_lower.contains("sempron") {
                    brand_arch(MicroArch::K8, "Paris", Some(N130))
                } else {
                    brand_arch(MicroArch::K8, "Newcastle", Some(N130))
                }
            }
            (0, 12 | 13) => brand_arch(MicroArch::K8, "Newcastle", Some(N130)),
            (0, 15) => brand_arch(MicroArch::K8, "Winchester", Some(N90)),

            // 90nm Venice / San Diego / Toledo / Manchester / Lancaster / Orleans / Windsor
            (1, 4) => brand_arch(MicroArch::K8, "Lancaster", Some(N90)),
            (1, 8) => {
                if m_lower.contains("opteron") {
                    brand_arch(MicroArch::K8, "Troy/Venus", Some(N90))
                } else {
                    brand_arch(MicroArch::K8, "San Diego", Some(N90))
                }
            }
            (1, 11) => brand_arch(MicroArch::K8, "San Diego", Some(N90)),
            (1, 15) => {
                if m_lower.contains("sempron") {
                    brand_arch(MicroArch::K8, "Palermo", Some(N90))
                } else {
                    brand_arch(MicroArch::K8, "Venice", Some(N90))
                }
            }
            (2, 1 | 3) => {
                if is_dual_core {
                    brand_arch(MicroArch::K8, "Toledo", Some(N90))
                } else {
                    brand_arch(MicroArch::K8, "San Diego", Some(N90))
                }
            }
            (2, 4) => brand_arch(MicroArch::K8, "Lancaster", Some(N90)),
            (2, 7) => brand_arch(MicroArch::K8, "San Diego", Some(N90)),
            (2, 11) => {
                if is_dual_core {
                    brand_arch(MicroArch::K8, "Manchester", Some(N90))
                } else {
                    brand_arch(MicroArch::K8, "Venice", Some(N90))
                }
            }
            (2, 15) => {
                if m_lower.contains("sempron") {
                    brand_arch(MicroArch::K8, "Palermo", Some(N90))
                } else {
                    brand_arch(MicroArch::K8, "Venice", Some(N90))
                }
            }
            (3, 7) => brand_arch(MicroArch::K8, "San Diego", Some(N90)),
            (3, 15) => brand_arch(MicroArch::K8, "Venice", Some(N90)),
            (4, 1 | 3) => {
                if m_lower.contains("opteron") {
                    brand_arch(MicroArch::K8, "Santa Rosa/Santa Ana", Some(N90))
                } else {
                    brand_arch(MicroArch::K8, "Windsor", Some(N90))
                }
            }
            (4, 8 | 11) => brand_arch(MicroArch::K8, "Windsor", Some(N90)),
            (4, 15) => {
                if m_lower.contains("sempron") {
                    brand_arch(MicroArch::K8, "Manila", Some(N90))
                } else {
                    brand_arch(MicroArch::K8, "Orleans", Some(N90))
                }
            }
            (5, 15) => brand_arch(MicroArch::K8, "Orleans", Some(N90)),

            // 65nm Brisbane / Sparta / Lima / Sherman
            (6, 8 | 11) => brand_arch(MicroArch::K8, "Brisbane", Some(N65)),
            (6, 15) => brand_arch(MicroArch::K8, "Sparta", Some(N65)),
            (7, 11) => brand_arch(MicroArch::K8, "Brisbane", Some(N65)),
            (7, 15) => brand_arch(MicroArch::K8, "Sparta", Some(N65)),
            (12, 1) => brand_arch(MicroArch::K8, "Sherman", Some(N65)),

            _ => brand_arch(MicroArch::K8, "Hammer", Some(N90)),
        }
    }

    #[cfg(not(dos))]
    fn disambiguate_k10(
        m_lower: &str,
        s: CpuSignature,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> CpuArch {
        let cores = amd_logical_cores();

        match (s.extended_model, s.model) {
            // 65nm Stars (Barcelona / Agena / Toliman / Kuma)
            (0, 2) => {
                if m_lower.contains("opteron") {
                    brand_arch(MicroArch::K10, "Barcelona", Some(N65))
                } else {
                    match cores {
                        2 => brand_arch(MicroArch::K10, "Kuma", Some(N65)),
                        3 => brand_arch(MicroArch::K10, "Toliman", Some(N65)),
                        _ => brand_arch(MicroArch::K10, "Agena", Some(N65)),
                    }
                }
            }

            // 45nm Stars (Shanghai / Deneb / Heka / Callisto)
            (0, 4) => {
                if m_lower.contains("opteron") {
                    brand_arch(MicroArch::K10, "Shanghai", Some(N45))
                } else {
                    match cores {
                        2 => brand_arch(MicroArch::K10, "Callisto", Some(N45)),
                        3 => brand_arch(MicroArch::K10, "Heka", Some(N45)),
                        _ => brand_arch(MicroArch::K10, "Deneb", Some(N45)),
                    }
                }
            }

            // 45nm Athlon II X3/X4 (Propus / Rana)
            (0, 5) => {
                if cores == 3 || m_lower.contains("x3") {
                    brand_arch(MicroArch::K10, "Rana", Some(N45))
                } else {
                    brand_arch(MicroArch::K10, "Propus", Some(N45))
                }
            }

            // 45nm Athlon II X2 / Sempron (Regor / Sargas)
            (0, 6) => {
                if cores == 1 || m_lower.contains("sempron") || m_lower.contains("sargas") {
                    brand_arch(MicroArch::K10, "Sargas", Some(N45))
                } else {
                    brand_arch(MicroArch::K10, "Regor", Some(N45))
                }
            }

            // 45nm 6-core Opteron (Istanbul)
            (0, 8) => brand_arch(MicroArch::K10, "Istanbul", Some(N45)),

            // 45nm Multi-die Opteron (Magny-Cours / Lisbon)
            (0, 9) => {
                if m_lower.contains("61") || cores >= 8 {
                    brand_arch(MicroArch::K10, "Magny-Cours", Some(N45))
                } else {
                    brand_arch(MicroArch::K10, "Lisbon", Some(N45))
                }
            }

            // 45nm Phenom II X6 / X4 (Thuban / Zosma)
            (0, 10) => {
                if cores == 4 || m_lower.contains("zosma") {
                    brand_arch(MicroArch::K10, "Zosma", Some(N45))
                } else {
                    brand_arch(MicroArch::K10, "Thuban", Some(N45))
                }
            }

            _ => brand_arch(MicroArch::K10, "Stars", Some(N45)),
        }
    }

    #[cfg(not(dos))]
    fn disambiguate_fam15h(
        m_lower: &str,
        s: CpuSignature,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> CpuArch {
        match (s.extended_model, s.model) {
            // Bulldozer (Zambezi / Interlagos / Valencia, 32nm)
            (0, 0 | 1) => {
                if m_lower.contains("opteron") {
                    brand_arch(MicroArch::Bulldozer, "Interlagos/Valencia", Some(N32))
                } else {
                    brand_arch(MicroArch::Bulldozer, "Zambezi", Some(N32))
                }
            }

            // Piledriver (Vishera / Abu Dhabi / Seoul, 32nm)
            (0, 2) => {
                if m_lower.contains("opteron") {
                    brand_arch(MicroArch::Piledriver, "Abu Dhabi/Seoul", Some(N32))
                } else {
                    brand_arch(MicroArch::Piledriver, "Vishera", Some(N32))
                }
            }

            // Piledriver APU (Trinity, 32nm)
            (1, 0) => brand_arch(MicroArch::Piledriver, "Trinity", Some(N32)),

            // Piledriver APU (Richland, 32nm)
            (1, 3) => brand_arch(MicroArch::Piledriver, "Richland", Some(N32)),

            // Steamroller APU (Kaveri / Godavari, 28nm)
            (3, 0) => brand_arch(MicroArch::Steamroller, "Kaveri", Some(N28)),
            (3, 8) => brand_arch(MicroArch::Steamroller, "Godavari", Some(N28)),

            // Excavator APU (Carrizo / Bristol Ridge / Stoney Ridge, 28nm)
            (6, 0) => brand_arch(MicroArch::Excavator, "Carrizo", Some(N28)),
            (6, 5) => brand_arch(MicroArch::Excavator, "Stoney Ridge", Some(N28)),
            (7, 0) => brand_arch(MicroArch::Excavator, "Bristol Ridge", Some(N28)),
            (7, 5) => brand_arch(MicroArch::Excavator, "Stoney Ridge", Some(N28)),

            _ => brand_arch(MicroArch::Bulldozer, "Bulldozer", Some(N32)),
        }
    }

    #[cfg(not(dos))]
    fn disambiguate_fam16h(
        m_lower: &str,
        s: CpuSignature,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> CpuArch {
        match (s.extended_model, s.model) {
            // Jaguar (Kabini / Temash, 28nm)
            (0, 0) => {
                if m_lower.contains("temash") {
                    brand_arch(MicroArch::Jaguar, "Temash", Some(N28))
                } else {
                    brand_arch(MicroArch::Jaguar, "Kabini", Some(N28))
                }
            }
            // Jaguar Server (Kyoto, 28nm)
            (0, 1) => brand_arch(MicroArch::Jaguar, "Kyoto", Some(N28)),

            // Jaguar Console SoCs (PS4 Liverpool / Xbox One Durango, 28nm)
            (2, 6) => brand_arch(MicroArch::Jaguar, "Liverpool/Durango", Some(N28)),

            // Puma 2014 (Beema / Mullins / Carrizo-L, 28nm)
            (3, 0) => {
                if m_lower.contains("carrizo") {
                    brand_arch(MicroArch::Puma2014, "Carrizo-L", Some(N28))
                } else if m_lower.contains("mullins") {
                    brand_arch(MicroArch::Puma2014, "Mullins", Some(N28))
                } else {
                    brand_arch(MicroArch::Puma2014, "Beema", Some(N28))
                }
            }

            _ => brand_arch(MicroArch::Jaguar, "Kabini", Some(N28)),
        }
    }

    #[cfg(not(dos))]
    fn disambiguate_fam17h(
        m_lower: &str,
        s: CpuSignature,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> CpuArch {
        match (s.extended_model, s.model) {
            // ----------------------------------------------------------------
            // Zen 1 / Zen+ (14nm / 12nm)
            // ----------------------------------------------------------------
            // Summit Ridge / Whitehaven / Naples (Zen, 14nm)
            (0, 1) => {
                if m_lower.contains("epyc") {
                    brand_arch(MicroArch::Zen, "Naples", Some(N14))
                } else if m_lower.contains("threadripper") {
                    brand_arch(MicroArch::Zen, "Whitehaven", Some(N14))
                } else {
                    brand_arch(MicroArch::Zen, "Summit Ridge", Some(N14))
                }
            }

            // Pinnacle Ridge / Colfax (Zen+, 12nm)
            (0, 8) => {
                if m_lower.contains("threadripper") {
                    brand_arch(MicroArch::ZenPlus, "Colfax", Some(N12))
                } else {
                    brand_arch(MicroArch::ZenPlus, "Pinnacle Ridge", Some(N12))
                }
            }

            // Raven Ridge / Dali / Pollock (Zen, 14nm)
            (1, 1) => {
                if m_lower.contains("dali")
                    || m_lower.contains("pollock")
                    || m_lower.contains("3020e")
                    || m_lower.contains("3050e")
                {
                    brand_arch(MicroArch::Zen, "Dali", Some(N14))
                } else {
                    brand_arch(MicroArch::Zen, "Raven Ridge", Some(N14))
                }
            }

            // Picasso (Zen+, 12nm APU)
            (1, 8) => brand_arch(MicroArch::ZenPlus, "Picasso", Some(N12)),

            // Dali / Pollock (Zen, 14nm)
            (2, 0) => brand_arch(MicroArch::Zen, "Dali/Pollock", Some(N14)),

            // ----------------------------------------------------------------
            // Zen 2 (7nm / 6nm)
            // ----------------------------------------------------------------
            // Rome / Castle Peak (EPYC 7002 / Threadripper 3000, 7nm)
            (3, 1) => {
                if m_lower.contains("threadripper") {
                    brand_arch(MicroArch::Zen2, "Castle Peak", Some(N7))
                } else {
                    brand_arch(MicroArch::Zen2, "Rome", Some(N7))
                }
            }

            // Xbox Series X/S (Flute) / PS5 (Oberon / Ariel) (Zen 2, 7nm)
            (4, 0) => brand_arch(MicroArch::Zen2, "Flute/Oberon", Some(N7)),

            // Renoir / Lucienne / Grey Hawk (Zen 2, 7nm APU)
            (6, 0) => {
                if m_lower.contains("5300u")
                    || m_lower.contains("5500u")
                    || m_lower.contains("5700u")
                    || m_lower.contains("lucienne")
                {
                    brand_arch(MicroArch::Zen2, "Lucienne", Some(N7))
                } else if m_lower.contains("grey hawk") {
                    brand_arch(MicroArch::Zen2, "Grey Hawk", Some(N7))
                } else {
                    brand_arch(MicroArch::Zen2, "Renoir", Some(N7))
                }
            }

            // Matisse (Ryzen 3000 Desktop, 7nm)
            (7, 1) => brand_arch(MicroArch::Zen2, "Matisse", Some(N7)),

            // Van Gogh / Aerith (Steam Deck APU, 7nm)
            (9, 0) => brand_arch(MicroArch::Zen2, "Van Gogh", Some(N7)),

            // Mendocino (Ryzen 7020 Mobile, 6nm)
            (10, 0) => brand_arch(MicroArch::Zen2, "Mendocino", Some(N6)),

            _ => brand_arch(MicroArch::Zen, "Zen", Some(N14)),
        }
    }

    #[cfg(not(dos))]
    fn disambiguate_fam19h(
        m_lower: &str,
        s: CpuSignature,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> CpuArch {
        match (s.extended_model, s.model) {
            // ----------------------------------------------------------------
            // Zen 3: Milan / Milan-X / Chagall (7nm Server / HEDT)
            // ----------------------------------------------------------------
            (0, 1) => {
                if m_lower.contains("threadripper") || m_lower.contains("pro 5") {
                    brand_arch(MicroArch::Zen3, "Chagall", Some(N7))
                } else if m_lower.contains("3d")
                    || m_lower.contains("7773x")
                    || m_lower.contains("7573x")
                {
                    brand_arch(MicroArch::Zen3, "Milan-X", Some(N7))
                } else {
                    brand_arch(MicroArch::Zen3, "Milan", Some(N7))
                }
            }

            // ----------------------------------------------------------------
            // Zen 4 / Zen 4c: Genoa / Genoa-X / Storm Peak / Bergamo / Siena (5nm)
            // ----------------------------------------------------------------
            (1, 1) => {
                if m_lower.contains("threadripper")
                    || (m_lower.contains("79") && m_lower.contains("wx"))
                {
                    brand_arch(MicroArch::Zen4, "Storm Peak", Some(N5))
                } else if m_lower.contains("3d")
                    || m_lower.contains("9684x")
                    || m_lower.contains("9384x")
                    || m_lower.contains("9184x")
                {
                    brand_arch(MicroArch::Zen4, "Genoa-X", Some(N5))
                } else {
                    brand_arch(MicroArch::Zen4, "Genoa", Some(N5))
                }
            }

            (1, 2) => {
                if m_lower.contains("8004") || m_lower.contains("siena") {
                    brand_arch(MicroArch::Zen4C, "Siena", Some(N5))
                } else {
                    brand_arch(MicroArch::Zen4C, "Bergamo", Some(N5))
                }
            }

            // ----------------------------------------------------------------
            // Zen 3: Vermeer / Vermeer-X (Ryzen 5000 Desktop, 7nm)
            // ----------------------------------------------------------------
            (2, 1) => {
                if m_lower.contains("x3d")
                    || m_lower.contains("5800x3d")
                    || m_lower.contains("5700x3d")
                    || m_lower.contains("5600x3d")
                {
                    brand_arch(MicroArch::Zen3, "Vermeer-X", Some(N7))
                } else {
                    brand_arch(MicroArch::Zen3, "Vermeer", Some(N7))
                }
            }

            // ----------------------------------------------------------------
            // Zen 3+: Rembrandt / Rembrandt-R (Yellow Carp, 6nm Mobile)
            // ----------------------------------------------------------------
            (4, 4) => {
                if m_lower.contains("7035")
                    || m_lower.contains("7535")
                    || m_lower.contains("7735")
                    || m_lower.contains("7435")
                    || m_lower.contains("7235")
                {
                    brand_arch(MicroArch::Zen3Plus, "Rembrandt-R", Some(N6))
                } else {
                    brand_arch(MicroArch::Zen3Plus, "Rembrandt", Some(N6))
                }
            }

            // ----------------------------------------------------------------
            // Zen 3: Cezanne / Barcelo / Barcelo-R (Ryzen 5000/7030 APU, 7nm)
            // ----------------------------------------------------------------
            (5, 0 | 1) => {
                if m_lower.contains("7030")
                    || m_lower.contains("7530")
                    || m_lower.contains("7730")
                    || m_lower.contains("7330")
                {
                    brand_arch(MicroArch::Zen3, "Barcelo-R", Some(N7))
                } else if m_lower.contains("5625")
                    || m_lower.contains("5825")
                    || m_lower.contains("5425")
                    || m_lower.contains("barcelo")
                {
                    brand_arch(MicroArch::Zen3, "Barcelo", Some(N7))
                } else {
                    brand_arch(MicroArch::Zen3, "Cezanne", Some(N7))
                }
            }

            // ----------------------------------------------------------------
            // Zen 4: Raphael / Raphael-X / Dragon Range (5nm)
            // ----------------------------------------------------------------
            (6, 1) => {
                if m_lower.contains("hx")
                    || m_lower.contains("7945")
                    || m_lower.contains("7845")
                    || m_lower.contains("7745")
                    || m_lower.contains("7645")
                {
                    brand_arch(MicroArch::Zen4, "Dragon Range", Some(N5))
                } else if m_lower.contains("x3d")
                    || m_lower.contains("7800x3d")
                    || m_lower.contains("7900x3d")
                    || m_lower.contains("7950x3d")
                {
                    brand_arch(MicroArch::Zen4, "Raphael-X", Some(N5))
                } else {
                    brand_arch(MicroArch::Zen4, "Raphael", Some(N5))
                }
            }

            // ----------------------------------------------------------------
            // Zen 4: Phoenix / Phoenix 2 (4nm APU)
            // ----------------------------------------------------------------
            (7, 4) => {
                if m_lower.contains("8500g")
                    || m_lower.contains("8300g")
                    || m_lower.contains("7540u")
                    || m_lower.contains("7440u")
                    || m_lower.contains("phoenix 2")
                {
                    brand_arch(MicroArch::Zen4, "Phoenix 2", Some(N4))
                } else {
                    brand_arch(MicroArch::Zen4, "Phoenix", Some(N4))
                }
            }

            // ----------------------------------------------------------------
            // Zen 4: Hawk Point (4nm Mobile)
            // ----------------------------------------------------------------
            (7, 8) => brand_arch(MicroArch::Zen4, "Hawk Point", Some(N4)),

            // ----------------------------------------------------------------
            // Zen 3: Trent / Badami (7nm Embedded)
            // ----------------------------------------------------------------
            (8, 0) => brand_arch(MicroArch::Zen3, "Trent", Some(N7)),

            _ => brand_arch(MicroArch::Zen3, "Zen 3", Some(N7)),
        }
    }

    #[cfg(not(dos))]
    fn disambiguate_fam1ah(
        m_lower: &str,
        s: CpuSignature,
        brand_arch: &impl Fn(MicroArch, &'static str, Option<&'static str>) -> CpuArch,
    ) -> CpuArch {
        match (s.extended_model, s.model) {
            // Turin (EPYC 9005 Zen 5, 4nm)
            (0, 1) => brand_arch(MicroArch::Zen5, "Turin", Some(N4)),

            // Turin Dense (EPYC 9005 Zen 5c, 3nm)
            (0, 2) => brand_arch(MicroArch::Zen5C, "Turin Dense", Some(N3)),

            // Strix Point (Ryzen AI 300 / Strix 1, Zen 5 + Zen 5c, 4nm)
            (2, 4) => brand_arch(MicroArch::Zen5, "Strix Point", Some(N4)),

            // Granite Ridge (Ryzen 9000 Desktop) / Fire Range (Mobile HX) (Zen 5, 4nm)
            (4, 4) => {
                if m_lower.contains("hx") || m_lower.contains("fire range") {
                    brand_arch(MicroArch::Zen5, "Fire Range", Some(N4))
                } else {
                    brand_arch(MicroArch::Zen5, "Granite Ridge", Some(N4))
                }
            }

            // Krackan Point (Kraken, Zen 5 + Zen 5c, 4nm)
            (6, 0) => brand_arch(MicroArch::Zen5, "Krackan Point", Some(N4)),

            // Strix Halo (Ryzen AI Max 300, Zen 5, 4nm)
            (7, 0) => brand_arch(MicroArch::Zen5, "Strix Halo", Some(N4)),

            _ => brand_arch(MicroArch::Zen5, "Zen 5", Some(N4)),
        }
    }
}

impl TMicroArch for Amd {
    fn micro_arch(model: &str, s: CpuSignature) -> CpuArch {
        let brand_arch = CpuArch::brand_arch(model, "AMD", VENDOR_AMD);

        if let Some(arch) = Self::legacy_micro_arch(s, &brand_arch) {
            return arch;
        }

        #[cfg(not(dos))]
        if let Some(arch) = Self::modern_micro_arch(model, s, &brand_arch) {
            return arch;
        }

        brand_arch(MicroArch::Unknown, UNK, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::x86::UNK;
    use crate::x86::micro_arch::tests::dummy_signature;

    #[test]
    fn test_cpu_arch_find_amd_classic() {
        // Am486DX2
        let sig = dummy_signature(4, 3, 0, 0, 0);
        let arch = Amd::micro_arch("Am486DX2", sig);
        assert_eq!(arch.micro_arch, MicroArch::Am486);
        assert_eq!(arch.code_name, "Am486DX2");

        // Am5x86WB
        let sig = dummy_signature(4, 15, 0, 0, 0);
        let arch = Amd::micro_arch("Am5x86", sig);
        assert_eq!(arch.micro_arch, MicroArch::Am5x86);
        assert_eq!(arch.code_name, "Am5x86WB");

        // K5 (SSA/5)
        let sig = dummy_signature(5, 0, 0, 0, 0);
        let arch = Amd::micro_arch("AMD K5", sig);
        assert_eq!(arch.micro_arch, MicroArch::SSA5);
        assert_eq!(arch.code_name, "SSA/5");
        assert_eq!(arch.technology, Some("350nm"));

        // K5 (5k86)
        let sig = dummy_signature(5, 1, 0, 0, 0);
        let arch = Amd::micro_arch("AMD-K5", sig);
        assert_eq!(arch.micro_arch, MicroArch::K5);
        assert_eq!(arch.code_name, "5k86");
        assert_eq!(arch.technology, Some("350nm"));

        // K6-2 (Chompers)
        let sig = dummy_signature(5, 8, 0, 0, 0);
        let arch = Amd::micro_arch("AMD-K6(tm)-2/450", sig);
        assert_eq!(arch.micro_arch, MicroArch::K6);
        assert_eq!(arch.code_name, "Chompers/CXT");
        assert_eq!(arch.technology, Some("250nm"));

        // K6-III (Sharptooth)
        let sig = dummy_signature(5, 9, 0, 0, 0);
        let arch = Amd::micro_arch("AMD-K6(tm)-III", sig);
        assert_eq!(arch.micro_arch, MicroArch::K6);
        assert_eq!(arch.code_name, "Sharptooth");
        assert_eq!(arch.technology, Some("250nm"));

        // Geode LX
        let sig = dummy_signature(5, 10, 0, 0, 0);
        let arch = Amd::micro_arch("AMD Geode(TM) LX 800", sig);
        assert_eq!(arch.micro_arch, MicroArch::Geode);
        assert_eq!(arch.code_name, "Geode LX");
        assert_eq!(arch.technology, Some("130nm"));
    }

    #[test]
    fn test_cpu_arch_find_amd_k7_k8() {
        // Athlon (Thunderbird)
        let sig = dummy_signature(6, 4, 0, 0, 2);
        let arch = Amd::micro_arch("AMD Athlon(tm) Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::K7);
        assert_eq!(arch.code_name, "Thunderbird");
        assert_eq!(arch.technology, Some("180nm"));

        // Athlon XP (Barton)
        let sig = dummy_signature(6, 10, 0, 0, 0);
        let arch = Amd::micro_arch("AMD Athlon(tm) XP 2800+", sig);
        assert_eq!(arch.micro_arch, MicroArch::K7);
        assert_eq!(arch.code_name, "Barton");
        assert_eq!(arch.technology, Some("130nm"));

        // Athlon 64 (Winchester)
        let sig = dummy_signature(15, 15, 0, 0, 0);
        let arch = Amd::micro_arch("AMD Athlon(tm) 64 Processor 3000+", sig);
        assert_eq!(arch.micro_arch, MicroArch::K8);
        assert_eq!(arch.code_name, "Winchester");
        assert_eq!(arch.technology, Some("90nm"));

        // Athlon 64 X2 (Windsor)
        let sig = dummy_signature(15, 11, 0, 4, 2);
        let arch = Amd::micro_arch("AMD Athlon(tm) 64 X2 Dual Core Processor 4200+", sig);
        assert_eq!(arch.micro_arch, MicroArch::K8);
        assert_eq!(arch.code_name, "Windsor");
        assert_eq!(arch.technology, Some("90nm"));

        // Athlon X2 (Brisbane)
        let sig = dummy_signature(15, 11, 0, 6, 2);
        let arch = Amd::micro_arch("AMD Athlon(tm) X2 Dual-Core QL-60", sig);
        assert_eq!(arch.micro_arch, MicroArch::K8);
        assert_eq!(arch.code_name, "Brisbane");
        assert_eq!(arch.technology, Some("65nm"));
    }

    #[test]
    fn test_cpu_arch_find_amd_k10_and_bulldozer() {
        // Phenom II X4 (Deneb)
        let sig = dummy_signature(15, 4, 1, 0, 2);
        let arch = Amd::micro_arch("AMD Phenom(tm) II X4 955 Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::K10);
        assert_eq!(arch.code_name, "Deneb");
        assert_eq!(arch.technology, Some("45nm"));

        // Phenom II X6 (Thuban)
        let sig = dummy_signature(15, 10, 1, 0, 0);
        let arch = Amd::micro_arch("AMD Phenom(tm) II X6 1090T Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::K10);
        assert_eq!(arch.code_name, "Thuban");
        assert_eq!(arch.technology, Some("45nm"));

        // Turion X2 Ultra (Griffin / Puma 2008)
        let sig = dummy_signature(15, 3, 2, 0, 1);
        let arch = Amd::micro_arch("AMD Turion(tm) X2 Ultra Dual-Core Mobile ZM-82", sig);
        assert_eq!(arch.micro_arch, MicroArch::Puma2008);
        assert_eq!(arch.code_name, "Griffin");
        assert_eq!(arch.technology, Some("65nm"));

        // Llano APU
        let sig = dummy_signature(15, 1, 3, 0, 0);
        let arch = Amd::micro_arch("AMD A8-3850 APU with Radeon(tm) HD Graphics", sig);
        assert_eq!(arch.micro_arch, MicroArch::K10);
        assert_eq!(arch.code_name, "Llano");
        assert_eq!(arch.technology, Some("32nm"));

        // Bobcat (Zacate)
        let sig = dummy_signature(15, 2, 5, 0, 0);
        let arch = Amd::micro_arch("AMD E-350 Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::Bobcat);
        assert_eq!(arch.code_name, "Zacate");
        assert_eq!(arch.technology, Some("40nm"));

        // Bulldozer (Zambezi FX-8150)
        let sig = dummy_signature(15, 1, 6, 0, 2);
        let arch = Amd::micro_arch("AMD FX(tm)-8150 Eight-Core Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::Bulldozer);
        assert_eq!(arch.code_name, "Zambezi");
        assert_eq!(arch.technology, Some("32nm"));

        // Piledriver (Vishera FX-8350)
        let sig = dummy_signature(15, 2, 6, 0, 0);
        let arch = Amd::micro_arch("AMD FX(tm)-8350 Eight-Core Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::Piledriver);
        assert_eq!(arch.code_name, "Vishera");
        assert_eq!(arch.technology, Some("32nm"));

        // Steamroller (Kaveri A10-7850K)
        let sig = dummy_signature(15, 0, 6, 3, 1);
        let arch = Amd::micro_arch("AMD A10-7850K APU with Radeon(R) R7 Graphics", sig);
        assert_eq!(arch.micro_arch, MicroArch::Steamroller);
        assert_eq!(arch.code_name, "Kaveri");
        assert_eq!(arch.technology, Some("28nm"));

        // Excavator (Bristol Ridge A12-9800)
        let sig = dummy_signature(15, 0, 6, 7, 1);
        let arch = Amd::micro_arch("AMD A12-9800 RADEON R7, 12 COMPUTE CORES 4C+8G", sig);
        assert_eq!(arch.micro_arch, MicroArch::Excavator);
        assert_eq!(arch.code_name, "Bristol Ridge");
        assert_eq!(arch.technology, Some("28nm"));

        // Jaguar (Kabini A4-5000)
        let sig = dummy_signature(15, 0, 7, 0, 1);
        let arch = Amd::micro_arch("AMD A4-5000 APU with Radeon(HD) Graphics", sig);
        assert_eq!(arch.micro_arch, MicroArch::Jaguar);
        assert_eq!(arch.code_name, "Kabini");
        assert_eq!(arch.technology, Some("28nm"));

        // Puma (Beema A6-6310)
        let sig = dummy_signature(15, 0, 7, 3, 1);
        let arch = Amd::micro_arch("AMD A6-6310 APU with AMD Radeon R4 Graphics", sig);
        assert_eq!(arch.micro_arch, MicroArch::Puma2014);
        assert_eq!(arch.code_name, "Beema");
        assert_eq!(arch.technology, Some("28nm"));
    }

    #[test]
    fn test_cpu_arch_find_amd_zen1_zen2() {
        // Zen 1: Summit Ridge (Ryzen 7 1800X)
        let sig = dummy_signature(15, 1, 8, 0, 1);
        let arch = Amd::micro_arch("AMD Ryzen 7 1800X Eight-Core Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen);
        assert_eq!(arch.code_name, "Summit Ridge");
        assert_eq!(arch.technology, Some("14nm"));

        // Zen 1: Naples (EPYC 7601)
        let sig = dummy_signature(15, 1, 8, 0, 2);
        let arch = Amd::micro_arch("AMD EPYC 7601 32-Core Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen);
        assert_eq!(arch.code_name, "Naples");
        assert_eq!(arch.technology, Some("14nm"));

        // Zen 1: Raven Ridge (Ryzen 5 2400G)
        let sig = dummy_signature(15, 1, 8, 1, 0);
        let arch = Amd::micro_arch("AMD Ryzen 5 2400G with Radeon Vega Graphics", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen);
        assert_eq!(arch.code_name, "Raven Ridge");
        assert_eq!(arch.technology, Some("14nm"));

        // Zen+: Pinnacle Ridge (Ryzen 7 2700X)
        let sig = dummy_signature(15, 8, 8, 0, 2);
        let arch = Amd::micro_arch("AMD Ryzen 7 2700X Eight-Core Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::ZenPlus);
        assert_eq!(arch.code_name, "Pinnacle Ridge");
        assert_eq!(arch.technology, Some("12nm"));

        // Zen+: Picasso (Ryzen 5 3500U)
        let sig = dummy_signature(15, 8, 8, 1, 1);
        let arch = Amd::micro_arch("AMD Ryzen 5 3500U with Radeon Vega Mobile Gfx", sig);
        assert_eq!(arch.micro_arch, MicroArch::ZenPlus);
        assert_eq!(arch.code_name, "Picasso");
        assert_eq!(arch.technology, Some("12nm"));

        // Zen 2: Matisse (Ryzen 9 3900X)
        let sig = dummy_signature(15, 1, 8, 7, 0);
        let arch = Amd::micro_arch("AMD Ryzen 9 3900X 12-Core Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen2);
        assert_eq!(arch.code_name, "Matisse");
        assert_eq!(arch.technology, Some("7nm"));

        // Zen 2: Rome (EPYC 7742)
        let sig = dummy_signature(15, 1, 8, 3, 0);
        let arch = Amd::micro_arch("AMD EPYC 7742 64-Core Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen2);
        assert_eq!(arch.code_name, "Rome");
        assert_eq!(arch.technology, Some("7nm"));

        // Zen 2: Castle Peak (Threadripper 3970X)
        let sig = dummy_signature(15, 1, 8, 3, 0);
        let arch = Amd::micro_arch("AMD Ryzen Threadripper 3970X 32-Core Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen2);
        assert_eq!(arch.code_name, "Castle Peak");
        assert_eq!(arch.technology, Some("7nm"));

        // Zen 2: Renoir (Ryzen 7 4700U)
        let sig = dummy_signature(15, 0, 8, 6, 1);
        let arch = Amd::micro_arch("AMD Ryzen 7 4700U with Radeon Graphics", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen2);
        assert_eq!(arch.code_name, "Renoir");
        assert_eq!(arch.technology, Some("7nm"));

        // Zen 2: Lucienne (Ryzen 7 5700U)
        let sig = dummy_signature(15, 0, 8, 6, 1);
        let arch = Amd::micro_arch("AMD Ryzen 7 5700U with Radeon Graphics", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen2);
        assert_eq!(arch.code_name, "Lucienne");
        assert_eq!(arch.technology, Some("7nm"));

        // Zen 2: Van Gogh (Steam Deck Aerith)
        let sig = dummy_signature(15, 0, 8, 9, 0);
        let arch = Amd::micro_arch("AMD Custom APU 0405", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen2);
        assert_eq!(arch.code_name, "Van Gogh");
        assert_eq!(arch.technology, Some("7nm"));

        // Zen 2: Mendocino (Ryzen 5 7520U)
        let sig = dummy_signature(15, 0, 8, 10, 0);
        let arch = Amd::micro_arch("AMD Ryzen 5 7520U with Radeon Graphics", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen2);
        assert_eq!(arch.code_name, "Mendocino");
        assert_eq!(arch.technology, Some("6nm"));
    }

    #[test]
    fn test_cpu_arch_find_amd_zen3_zen4_zen5() {
        // Zen 3: Vermeer (Ryzen 9 5950X)
        let sig = dummy_signature(15, 1, 10, 2, 0);
        let arch = Amd::micro_arch("AMD Ryzen 9 5950X 16-Core Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen3);
        assert_eq!(arch.code_name, "Vermeer");
        assert_eq!(arch.technology, Some("7nm"));

        // Zen 3: Vermeer-X (Ryzen 7 5800X3D)
        let sig = dummy_signature(15, 1, 10, 2, 2);
        let arch = Amd::micro_arch("AMD Ryzen 7 5800X3D 8-Core Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen3);
        assert_eq!(arch.code_name, "Vermeer-X");
        assert_eq!(arch.technology, Some("7nm"));

        // Zen 3: Milan (EPYC 7763)
        let sig = dummy_signature(15, 1, 10, 0, 1);
        let arch = Amd::micro_arch("AMD EPYC 7763 64-Core Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen3);
        assert_eq!(arch.code_name, "Milan");
        assert_eq!(arch.technology, Some("7nm"));

        // Zen 3: Cezanne (Ryzen 7 5700G)
        let sig = dummy_signature(15, 0, 10, 5, 0);
        let arch = Amd::micro_arch("AMD Ryzen 7 5700G with Radeon Graphics", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen3);
        assert_eq!(arch.code_name, "Cezanne");
        assert_eq!(arch.technology, Some("7nm"));

        // Zen 3: Barcelo-R (Ryzen 7 7730U)
        let sig = dummy_signature(15, 0, 10, 5, 1);
        let arch = Amd::micro_arch("AMD Ryzen 7 7730U with Radeon Graphics", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen3);
        assert_eq!(arch.code_name, "Barcelo-R");
        assert_eq!(arch.technology, Some("7nm"));

        // Zen 3+: Rembrandt (Ryzen 7 6800H)
        let sig = dummy_signature(15, 4, 10, 4, 1);
        let arch = Amd::micro_arch("AMD Ryzen 7 6800H with Radeon Graphics", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen3Plus);
        assert_eq!(arch.code_name, "Rembrandt");
        assert_eq!(arch.technology, Some("6nm"));

        // Zen 4: Raphael (Ryzen 9 7950X)
        let sig = dummy_signature(15, 1, 10, 6, 2);
        let arch = Amd::micro_arch("AMD Ryzen 9 7950X 16-Core Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen4);
        assert_eq!(arch.code_name, "Raphael");
        assert_eq!(arch.technology, Some("5nm"));

        // Zen 4: Raphael-X (Ryzen 9 7950X3D)
        let sig = dummy_signature(15, 1, 10, 6, 2);
        let arch = Amd::micro_arch("AMD Ryzen 9 7950X3D 16-Core Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen4);
        assert_eq!(arch.code_name, "Raphael-X");
        assert_eq!(arch.technology, Some("5nm"));

        // Zen 4: Dragon Range (Ryzen 9 7945HX)
        let sig = dummy_signature(15, 1, 10, 6, 2);
        let arch = Amd::micro_arch("AMD Ryzen 9 7945HX with Radeon Graphics", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen4);
        assert_eq!(arch.code_name, "Dragon Range");
        assert_eq!(arch.technology, Some("5nm"));

        // Zen 4: Genoa (EPYC 9654)
        let sig = dummy_signature(15, 1, 10, 1, 1);
        let arch = Amd::micro_arch("AMD EPYC 9654 96-Core Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen4);
        assert_eq!(arch.code_name, "Genoa");
        assert_eq!(arch.technology, Some("5nm"));

        // Zen 4c: Bergamo (EPYC 9754)
        let sig = dummy_signature(15, 2, 10, 1, 1);
        let arch = Amd::micro_arch("AMD EPYC 9754 128-Core Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen4C);
        assert_eq!(arch.code_name, "Bergamo");
        assert_eq!(arch.technology, Some("5nm"));

        // Zen 4: Phoenix (Ryzen 7 7840HS)
        let sig = dummy_signature(15, 4, 10, 7, 1);
        let arch = Amd::micro_arch("AMD Ryzen 7 7840HS w/ Radeon 780M Graphics", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen4);
        assert_eq!(arch.code_name, "Phoenix");
        assert_eq!(arch.technology, Some("4nm"));

        // Zen 4: Hawk Point (Ryzen 7 8845HS)
        let sig = dummy_signature(15, 8, 10, 7, 1);
        let arch = Amd::micro_arch("AMD Ryzen 7 8845HS w/ Radeon 780M Graphics", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen4);
        assert_eq!(arch.code_name, "Hawk Point");
        assert_eq!(arch.technology, Some("4nm"));

        // Zen 5: Granite Ridge (Ryzen 9 9950X)
        let sig = dummy_signature(15, 4, 11, 4, 0);
        let arch = Amd::micro_arch("AMD Ryzen 9 9950X 16-Core Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen5);
        assert_eq!(arch.code_name, "Granite Ridge");
        assert_eq!(arch.technology, Some("4nm"));

        // Zen 5: Turin (EPYC 9655)
        let sig = dummy_signature(15, 1, 11, 0, 0);
        let arch = Amd::micro_arch("AMD EPYC 9655 96-Core Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen5);
        assert_eq!(arch.code_name, "Turin");
        assert_eq!(arch.technology, Some("4nm"));

        // Zen 5c: Turin Dense (EPYC 9755)
        let sig = dummy_signature(15, 2, 11, 0, 0);
        let arch = Amd::micro_arch("AMD EPYC 9755 128-Core Processor", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen5C);
        assert_eq!(arch.code_name, "Turin Dense");
        assert_eq!(arch.technology, Some("3nm"));

        // Zen 5: Strix Point (Ryzen AI 9 HX 370)
        let sig = dummy_signature(15, 4, 11, 2, 0);
        let arch = Amd::micro_arch("AMD Ryzen AI 9 HX 370 with Radeon 890M", sig);
        assert_eq!(arch.micro_arch, MicroArch::Zen5);
        assert_eq!(arch.code_name, "Strix Point");
        assert_eq!(arch.technology, Some("4nm"));

        // Unknown AMD Signature
        let sig_unknown = dummy_signature(99, 0, 0, 0, 0);
        let arch = Amd::micro_arch("AMD Unknown Processor", sig_unknown);
        assert_eq!(arch.micro_arch, MicroArch::Unknown);
        assert_eq!(arch.code_name, UNK);
    }
}
