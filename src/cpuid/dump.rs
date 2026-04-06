use super::Str;
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
    let result = cpuid::x86_cpuid_count(leaf, sub_leaf);
    let prefix = repeat_spaces(indent);
    let _ = writeln!(
        f,
        "{}0x{:08X} 0x{:02X}: eax=0x{:08X} ebx=0x{:08X} ecx=0x{:08X} edx=0x{:08X}",
        prefix, leaf, sub_leaf, result.eax, result.ebx, result.ecx, result.edx
    );
}

fn dump_cpu(f: &mut impl Write, cpu_idx: usize) {
    let _ = writeln!(f, "CPU {}:", cpu_idx);

    let max_leaf = cpuid::max_leaf();
    for leaf in 0..=max_leaf {
        dump_leaf(f, leaf, 0, 4);
    }

    let max_ext_leaf = cpuid::max_extended_leaf();
    for leaf in EXT_LEAF_0..=max_ext_leaf {
        dump_leaf(f, leaf, 0, 4);
    }

    let vendor = cpuid::vendor_str();

    let easter_egg = cpuid::Cpu::detect().easter_egg;
    if easter_egg.is_some() {
        #[allow(unreachable_patterns)]
        match &*vendor {
            VENDOR_AMD => dump_leaf(f, 0x8FFF_FFFF, 0, 4),
            #[cfg(target_arch = "x86")]
            _ => (),
            _ => (),
        }
    }

    if vendor == "CentaurHauls" || vendor == "Zhaoxin" {
        let max_centaur_leaf = cpuid::x86_cpuid(CENTAUR_LEAF_0).eax;
        for leaf in CENTAUR_LEAF_0..=max_centaur_leaf {
            dump_leaf(f, leaf, 0, 4);
        }
    } else if vendor == "GenuineTMx86" || vendor == "TransmetaCPU" {
        let max_transmeta_leaf = cpuid::x86_cpuid(TRANSMETA_LEAF_0).eax;
        for leaf in TRANSMETA_LEAF_0..=max_transmeta_leaf {
            dump_leaf(f, leaf, 0, 4);
        }
    }
}

pub fn dump_main() {
    #[cfg(target_os = "none")]
    use crate::print;

    let mut output: Str<8192> = Str::new();

    let logical_cores = cpuid::logical_cores() as usize;
    for i in 0..logical_cores {
        dump_cpu(&mut output, i);
    }

    print!("{}", output);
}
