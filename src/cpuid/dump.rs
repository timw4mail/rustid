use super::*;
use super::{CENTAUR_LEAF_0, EXT_LEAF_0, TRANSMETA_LEAF_0, VENDOR_AMD};
use crate::common::TCpu;
use crate::cpuid;
use core::fmt::Write;

fn repeat_spaces(n: usize) -> &'static str {
    const SPACES: &str = "                                        ";
    if n < SPACES.len() {
        &SPACES[..n]
    } else {
        SPACES
    }
}

fn dump_leaf(f: &mut impl Write, leaf: u32, sub_leaf: u32, indent: usize) {
    let result = x86_cpuid_count(leaf, sub_leaf);
    let prefix = repeat_spaces(indent);
    let _ = writeln!(
        f,
        "{}0x{:08X} 0x{:02X}: eax=0x{:08X} ebx=0x{:08X} ecx=0x{:08X} edx=0x{:08X}",
        prefix, leaf, sub_leaf, result.eax, result.ebx, result.ecx, result.edx
    );
}

fn dump_leaf_maybe_subleaves(f: &mut impl Write, leaf: u32, indent: usize) {
    match leaf {
        LEAF_4 | LEAF_18 | EXT_LEAF_1D => {
            let mut sub_leaf = 0;
            loop {
                let res = x86_cpuid_count(leaf, sub_leaf);
                if (res.eax & 0x1F) == 0 {
                    if sub_leaf == 0 {
                        dump_leaf(f, leaf, sub_leaf, indent);
                    }
                    break;
                }
                dump_leaf(f, leaf, sub_leaf, indent);
                sub_leaf += 1;
            }
        }
        LEAF_7 | LEAF_14 | LEAF_17 => {
            let res = x86_cpuid_count(leaf, 0);
            let max_subleaf = res.eax;
            for sub_leaf in 0..=max_subleaf {
                dump_leaf(f, leaf, sub_leaf, indent);
            }
        }
        LEAF_0B | LEAF_1F | EXT_LEAF_26 => {
            let mut sub_leaf = 0;
            loop {
                let res = x86_cpuid_count(leaf, sub_leaf);
                // Level type is ECX[15:8]. If 0, it's invalid/end.
                if (res.ecx & 0xFF00) == 0 {
                    if sub_leaf == 0 {
                        dump_leaf(f, leaf, sub_leaf, indent);
                    }
                    break;
                }
                dump_leaf(f, leaf, sub_leaf, indent);
                sub_leaf += 1;
            }
        }
        LEAF_0D => {
            let res0 = x86_cpuid_count(leaf, 0);
            let mask = (res0.edx as u64) << 32 | (res0.eax as u64);

            dump_leaf(f, leaf, 0, indent);
            dump_leaf(f, leaf, 1, indent);

            for sub_leaf in 2..64 {
                if (mask & (1u64 << sub_leaf)) != 0 {
                    dump_leaf(f, leaf, sub_leaf, indent);
                }
            }
        }
        _ => dump_leaf(f, leaf, 0, indent),
    }
}

pub fn dump_cpu(f: &mut impl Write, cpu_idx: usize) {
    let _ = writeln!(f, "CPU {}:", cpu_idx);

    let max_leaf = max_leaf();
    for leaf in 0..=max_leaf {
        dump_leaf_maybe_subleaves(f, leaf, 4);
    }

    if is_hypervisor_guest() {
        let max_hyp_leaf = max_hypervisor_leaf();
        for leaf in HYP_LEAF_0..=max_hyp_leaf {
            dump_leaf_maybe_subleaves(f, leaf, 4);
        }
    }

    let max_ext_leaf = max_extended_leaf();
    for leaf in EXT_LEAF_0..=max_ext_leaf {
        dump_leaf_maybe_subleaves(f, leaf, 4);
    }

    let vendor = vendor_str();

    let easter_egg = Cpu::detect().easter_egg;
    if easter_egg.is_some() {
        match &*vendor {
            VENDOR_AMD => dump_leaf(f, AMD_EASTER_EGG_ADDR, 0, 4),
            #[cfg(target_arch = "x86")]
            VENDOR_RISE | VENDOR_SIS | VENDOR_DMP | VENDOR_RDC => {
                dump_leaf(f, RISE_EASTER_EGG_ADDR, 0, 4)
            }
            _ => (),
        }
    }

    if vendor == "CentaurHauls" || vendor == "Zhaoxin" {
        let max_centaur_leaf = cpuid::x86_cpuid(CENTAUR_LEAF_0).eax;
        for leaf in CENTAUR_LEAF_0..=max_centaur_leaf {
            dump_leaf_maybe_subleaves(f, leaf, 4);
        }
    } else if vendor == "GenuineTMx86" || vendor == "TransmetaCPU" {
        let max_transmeta_leaf = cpuid::x86_cpuid(TRANSMETA_LEAF_0).eax;
        for leaf in TRANSMETA_LEAF_0..=max_transmeta_leaf {
            dump_leaf_maybe_subleaves(f, leaf, 4);
        }
    }
}
