//! CPU detection and information for x86/x86_64 processors.

use super::brand::CpuBrand;
use super::micro_arch::{CpuArch, MicroArch};
use super::topology::Topology;
use super::vendor::Cyrix;
use super::*;
#[cfg(std_os)]
use crate::common::{Cache, Speed};
use crate::common::{CoreType, DataSource, TDetect, UNK};
use alloc::string::String;
use alloc::vec::Vec;

#[cfg(std_os)]
use super::provider;

/// CPU feature class/level enumeration.
///
/// Represents the instruction set and feature level of an x86 processor,
/// roughly based on x86-64 microarchitecture levels.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FeatureClass {
    /// 80386-class processor
    i386,
    /// 80486-class processor
    i486,
    /// Pentium-class processor (i586)
    i586,
    /// Pentium Pro/II/III-class processor (i686)
    i686,
    /// i686 with SSE instruction
    i686_SSE,
    /// i686 with SSE2 instruction
    i686_SSE2,
    /// i686 with SSE3 instruction
    i686_SSE3,
    /// x86-64 version 1 (baseline SSE/SSE2)
    x86_64_v1,
    /// x86-64 version 2 (adds CMPXCHG16B, POPCNT, SSE4.2)
    x86_64_v2,
    /// x86-64 version 3 (adds AVX, AVX2, BMI, F16C, FMA)
    x86_64_v3,
    /// x86-64 version 4 (adds AVX-512)
    x86_64_v4,
}

impl FeatureClass {
    /// Cpu Feature Detection
    ///
    /// Roughly based on <https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels>
    pub fn detect() -> FeatureClass {
        use super::*;

        if has_avx512_f() {
            return FeatureClass::x86_64_v4;
        }

        if has_avx() && has_avx2() && has_bmi1() && has_bmi2() && has_f16c() && has_fma() {
            return FeatureClass::x86_64_v3;
        }

        if has_cx16() && has_popcnt() && has_sse3() && has_sse41() && has_sse42() && has_ssse3() {
            return FeatureClass::x86_64_v2;
        }

        if has_amd64() {
            return FeatureClass::x86_64_v1;
        }

        #[cfg(target_arch = "x86")]
        if is_cyrix() {
            return Cyrix::get_feature_class();
        }

        if has_sse3() {
            return FeatureClass::i686_SSE3;
        }

        if has_sse2() {
            return FeatureClass::i686_SSE2;
        }

        if has_sse() {
            return FeatureClass::i686_SSE;
        }

        if has_cmov() {
            return FeatureClass::i686;
        }

        if has_cx8() {
            return FeatureClass::i586;
        }

        if has_cpuid() && CpuSignature::detect().family == 4 {
            return FeatureClass::i486;
        }

        if is_486() {
            return FeatureClass::i486;
        }

        FeatureClass::i386
    }

    /// Returns a string representation of the feature class.
    pub fn to_str(self) -> &'static str {
        match self {
            FeatureClass::i386 => "i386",
            FeatureClass::i486 => "i486",
            FeatureClass::i586 => "i586",
            FeatureClass::i686 => "i686",
            FeatureClass::i686_SSE => "i686-SSE",
            FeatureClass::i686_SSE2 => "i686-SSE2",
            FeatureClass::i686_SSE3 => "i686-SSE3",
            FeatureClass::x86_64_v1 => "x86_64-v1",
            FeatureClass::x86_64_v2 => "x86_64-v2",
            FeatureClass::x86_64_v3 => "x86_64-v3",
            FeatureClass::x86_64_v4 => "x86_64-v4",
        }
    }
}

/// CPU signature containing family, model, and stepping information.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct CpuSignature {
    /// Extended family value from CPUID
    pub extended_family: u32,
    /// Family value from CPUID
    pub family: u32,
    /// Extended model value from CPUID
    pub extended_model: u32,
    /// Model value from CPUID
    pub model: u32,
    /// Stepping value from CPUID
    pub stepping: u32,
    /// Display family (calculated from family and extended_family)
    pub display_family: u32,
    /// Display model (calculated from model and extended_model)
    pub display_model: u32,
    /// Is this an Intel Overdrive CPU?
    pub is_overdrive: bool,
    /// Where did this signature information come from?
    pub source: DataSource,
}

impl CpuSignature {
    pub fn new(
        extended_family: u32,
        family: u32,
        extended_model: u32,
        model: u32,
        stepping: u32,
        source: DataSource,
    ) -> Self {
        let display_family = if family == 0xF {
            family + extended_family
        } else {
            family
        };

        let display_model = if family == 0x6 || family == 0xF {
            (extended_model << 4) + model
        } else {
            model
        };

        let is_overdrive = super::is_overdrive();

        Self {
            extended_family,
            family,
            extended_model,
            model,
            stepping,
            display_family,
            display_model,
            is_overdrive,
            source,
        }
    }

    pub fn new_synth(family: u32, model: u32, stepping: u32, source: DataSource) -> Self {
        Self::new(0, family, 0, model, stepping, source)
    }

    /// Detects the CPU signature from CPUID leaf 1.
    pub fn detect() -> Self {
        #[cfg(dos_os)]
        if !has_cpuid() {
            use super::vendor::cyrix::Cyrix;

            if super::is_cyrix() {
                if Cyrix::detect().dir0 > 0x13 {
                    let mut sig = Cyrix::get_signature_from_device_id();
                    if sig != CpuSignature::default() {
                        sig.source = DataSource::CpuMsr;
                        return sig;
                    }
                }
            }

            #[cfg(dos_real)]
            if let Some(mut reset_sig) = super::get_reset_signature() {
                reset_sig.source = DataSource::CpuReset;
                return reset_sig;
            }
        }

        let res = x86_cpuid(LEAF_1);
        let stepping = res.eax & 0xF;
        let model = (res.eax >> 4) & 0xF;
        let family = (res.eax >> 8) & 0xF;
        let extended_model = (res.eax >> 16) & 0xF;
        let extended_family = (res.eax >> 20) & 0xFF;

        Self::new(
            extended_family,
            family,
            extended_model,
            model,
            stepping,
            cpuid_data_source(),
        )
    }
}

/// x86 architecture-specific data.
#[derive(Debug, Default, PartialEq)]
pub struct X86Data {
    /// Does this cpu have cpuid instruction support
    pub has_cpuid: bool,
    /// CPU architecture and microarchitecture details
    pub arch: CpuArch,
    /// Hypervisor vendor string
    pub hyp_vendor_str: Option<String>,
    /// Easter egg string (hidden CPU info for some AMD/Rise processors)
    pub easter_egg: Option<String>,
    /// Model brand id
    pub brand_id: u32,
    /// CPU signature (family, model, stepping)
    pub signature: CpuSignature,
    /// Speed, threads, cores, sockets
    pub topology: Topology,
}

pub type CpuCore = crate::common::CpuCore<MicroArch>;
pub type Cpu = crate::common::Cpu<X86Data, MicroArch>;

impl Cpu {
    /// Gets the CPU model string.
    pub fn raw_model_string() -> String {
        read_multi_leaf_str(EXT_LEAF_2, EXT_LEAF_4)
    }

    #[cfg(not(dos_real))]
    fn intel_brand_index(&self) -> Option<&'static str> {
        let brand_id = get_brand_id();

        const CELERON: &str = "Intel(R) Celeron(R) processor";
        const XEON: &str = "Intel(R) Xeon(R) processor";
        const XEON_MP: &str = "Intel(R) Xeon(R) processor MP";

        let (family, model, stepping) = (
            self.signature.family,
            self.signature.model,
            self.signature.stepping,
        );

        // If the family and model are greater than (0xF, 0x3),
        // (Prescott, or 64-bit), this table dos not apply
        if family == 15 && model >= 3 {
            return None;
        }

        let str = match brand_id {
            0x01 | 0x0A | 0x14 => CELERON,
            0x02 | 0x04 => "Intel(R) Pentium(R) III processor",
            0x03 => match (family, model, stepping) {
                (0x6, 0xB, 0x1) => CELERON,
                _ => "Intel(R) Pentium(R) III Xeon",
            },
            0x06 => "Mobile Intel(R) Pentium(R) III processor-M",
            0x07 | 0x0F | 0x13 | 0x17 => "Mobile Intel(R) Celeron(R) processor",
            0x08 | 0x09 => "Intel(R) Pentium(R) 4 processor",
            0x0B => match (family, model, stepping) {
                (0xF, 0x1, 0x3) => XEON_MP,
                _ => XEON,
            },
            0x0C => XEON_MP,
            0x0E => match (family, model, stepping) {
                (0xF, 0x1, 0x3) => XEON,
                _ => "Mobile Intel(R) Pentium(R) 4 processor-M",
            },
            0x11 | 0x15 => "Mobile Genuine Intel(R) processor",
            0x12 => "Intel(R) Celeron(R) M processor",
            0x16 => "Intel(R) Pentium(R) M processor",
            _ => UNK,
        };

        match str {
            UNK => None,
            _ => Some(str),
        }
    }

    #[cfg(not(dos_real))]
    fn cleanup_model_string(s: &str) -> String {
        let str = s.replace("CPU", "");

        // Single-pass: build result without intermediate Vec
        let mut result = String::with_capacity(str.len());
        for part in str.split_ascii_whitespace() {
            if !part.is_empty() {
                if !result.is_empty() {
                    result.push(' ');
                }
                result.push_str(part);
            }
        }

        // Remove speed suffix after '@'
        if let Some(idx) = result.find('@') {
            result.truncate(idx);
            while result.ends_with(' ') {
                result.pop();
            }
        }

        result
    }

    /// Returns a human-readable display name for the CPU model.
    ///
    /// This attempts to produce a marketing-style name based on the
    /// detected CPU, falling back to architecture class names for
    /// older or unrecognized processors.
    pub fn display_model_string(&self) -> String {
        let brand = if !self.arch.vendor_string.is_empty() && self.arch.vendor_string != UNK {
            CpuBrand::from(self.arch.vendor_string.as_str())
        } else {
            CpuBrand::detect()
        };

        match brand {
            CpuBrand::AMD
                // The Geode NX is special
                if CpuBrand::detect() == CpuBrand::AMD
                    && self.signature.family == 6
                    && self.signature.model == 8
                    && self.signature.stepping == 1
                => {
                    return String::from("AMD Geode NX");
                }
            CpuBrand::Cyrix => {
                // Cyrix MSR model lookup is more accurate than the 'generic' way
                return Cyrix::model_string();
            }
            CpuBrand::Intel => {
                // Check the Intel model lookup table
                #[cfg(not(dos_real))]
                if let Some(model_name) = self.intel_brand_index() {
                    return String::from(model_name);
                }
            }
            CpuBrand::SiS => return String::from("SiS 550/551/552 SoC"),
            CpuBrand::Unknown => 'nocpuid: {
                // Not a 386 or 486
                if self.arch.model != UNK || self.signature.family > 4 {
                    break 'nocpuid;
                }

                // 486s without cpuid
                let s = if is_386() {
                    "'Classic' 386"
                } else {
                    match (self.signature.family, self.signature.model) {
                        (4, 2) => "'Classic' 486 SX",
                        (4, 3) => "'Classic' 486 DX2",
                        (4, 4) => "Intel 486SL",
                        (4, 5) => "'Classic' 486 SX2",
                        _ => "'Classic' 486",
                    }
                };

                return String::from(s);
            }
            _ => (),
        }

        let s = match self.arch.micro_arch {
            // AMD
            MicroArch::Am486 => match self.arch.code_name {
                "Am486DX" => "AMD 486 DX",
                "Am486DX-40" => "AMD 486 DX-40",
                "Am486SX" => "AMD 486 SX",
                "Am486DX2" => "AMD 486 DX2",
                "Am486X2WB" => "AMD 486 DX2 with Write-Back Cache",
                "Am486DX4" => "AMD 486 DX4",
                "Am486DX4WB" => "AMD 486 DX4 with Write-Back Cache",
                _ => "'Classic' 486",
            },
            MicroArch::Am5x86 => match self.arch.code_name {
                "Am5x86WB" => "AMD 5x86 with Write-Back Cache",
                _ => "AMD 5x86",
            },
            MicroArch::SSA5 => "AMD K5",

            // Centaur
            MicroArch::Winchip => "IDT Winchip",
            MicroArch::Winchip2 => "IDT Winchip 2",
            MicroArch::Winchip2A => "IDT Winchip 2A",
            MicroArch::Winchip2B => "IDT Winchip 2B",
            MicroArch::Winchip3 => "IDT Winchip 3",
            MicroArch::Samuel
            | MicroArch::Samuel2
            | MicroArch::Ezra
            | MicroArch::EzraT
            | MicroArch::Nehemiah => "VIA C3",
            MicroArch::Esther => "VIA C7",
            MicroArch::Isaiah => {
                if self.arch.model.contains("Eden") {
                    &self.arch.model.replace("Eden", "Nano")
                } else {
                    &self.arch.model
                }
            }

            //Intel
            MicroArch::RapidCad => "Intel RapidCAD",
            MicroArch::I486 => match self.arch.code_name {
                "i80486DX" => "Intel 486 DX",
                "i80486DX-50" => "Intel 486 DX-50",
                "i80486SX" => "Intel 486 SX",
                "i80486DX2" => "Intel 486 DX2",
                "i80486SL" => "Intel 486 SL",
                "i80486SX2" => "Intel 486 SX2",
                "i80486DX2WB" => "Intel 486 DX2 with Write-Back Cache",
                "i80486DX4" => "Intel 486 DX4",
                "i80486DX4WB" => "Intel 486 DX4 with Write-Back Cache",
                _ => "'Classic' 486",
            },
            MicroArch::P5 => {
                if has_mmx() {
                    "Intel Pentium with MMX"
                } else {
                    match self.arch.code_name {
                        "P24T" => "Intel Pentium Overdrive",
                        _ => "Intel Pentium",
                    }
                }
            }
            MicroArch::PentiumPro => "Intel Pentium Pro",
            MicroArch::PentiumII => "Intel Pentium II",
            MicroArch::PentiumIII => "Intel Pentium III",

            // Rise
            MicroArch::MP6 => match self.arch.code_name {
                "Lynx" => "Rise iDragon",
                _ => "Rise mP6",
            },

            // UMC
            MicroArch::U5S => "UMC Green CPU U5S (486 SX)",
            MicroArch::U5D => "UMC Green CPU U5D (486 DX)",

            // Make sure to return the original model string if there are no overrides
            _ => {
                if self.arch.model != UNK {
                    &self.arch.model
                } else {
                    UNK
                }
            }
        };

        #[cfg(not(dos_real))]
        return Self::cleanup_model_string(s);

        #[cfg(dos_real)]
        String::from(s)
    }

    pub(crate) fn easter_egg() -> Option<String> {
        let mut out: String = String::new();
        let brand = CpuBrand::detect();

        let addr = match brand {
            CpuBrand::AMD => AMD_EASTER_EGG_ADDR,
            CpuBrand::Rise | CpuBrand::SiS | CpuBrand::DMP | CpuBrand::Rdc => RISE_EASTER_EGG_ADDR,

            _ => 1,
        };

        if addr != 1 {
            let res = x86_cpuid(addr);

            let reg_list = match brand {
                // Surely there had to be a reason for this silly ordering?
                CpuBrand::Rise | CpuBrand::SiS => [res.ebx, res.edx, res.ecx, res.eax],

                _ => [res.eax, res.ebx, res.ecx, res.edx],
            };

            for &reg in &reg_list {
                let bytes = reg.to_le_bytes();
                for &b in &bytes {
                    if b != 0 {
                        out.push(b as char);
                    }
                }
            }
        }

        let trimmed = out.trim();
        if !trimmed.is_empty() {
            Some(String::from(trimmed))
        } else {
            None
        }
    }
}

impl TDetect for Cpu {
    /// Detects and returns comprehensive CPU information.
    ///
    /// Performs full CPU detection including architecture, microarchitecture,
    /// brand string, signature, features, and topology, enriching with OS
    /// information on live hardware.
    fn detect() -> Self {
        let mut cpu = Self::detect_cpuid();

        #[cfg(std_os)]
        if provider::info_source() == provider::CpuidInfoSource::Cpu {
            super::os::enrich_cpu(&mut cpu);
        }

        #[cfg(uefi)]
        super::efi::enrich_cpu(&mut cpu);

        #[cfg(dos_os)]
        super::dos::enrich_cpu(&mut cpu);

        cpu
    }
}

impl Cpu {
    /// Detects and returns comprehensive CPU information purely from CPUID leaves.
    ///
    /// This method guarantees that no operating system information (system name,
    /// OS socket counts, core pinning, or dynamic timer measurement) is queried.
    #[must_use]
    pub fn detect_cpuid() -> Self {
        let sig = CpuSignature::detect();
        let arch = CpuArch::find(&Self::raw_model_string(), sig, &vendor_str());
        let topology = Topology::detect_cpuid();
        let cores = Self::detect_cpuid_core_types(&arch, &topology);

        let extra = X86Data {
            has_cpuid: (is_cyrix() && Cyrix::can_enable_cpuid()) || has_cpuid(),
            arch,
            hyp_vendor_str: if is_hypervisor_guest() && max_hypervisor_leaf() > 0 {
                Some(hypervisor_str())
            } else {
                None
            },
            easter_egg: Self::easter_egg(),
            brand_id: get_brand_id(),
            signature: sig,
            topology,
        };

        Self {
            system: None,
            vendor: String::from(extra.arch.brand_name),
            model: extra.arch.model.clone(),
            cores,
            features: get_feature_list(),
            extra,
        }
    }

    /// Detects core types purely from CPUID contexts (if multiple dump contexts exist),
    /// or returns the single homogeneous cluster fallback.
    #[must_use]
    pub fn detect_cpuid_core_types(arch: &CpuArch, topology: &Topology) -> Vec<CpuCore> {
        #[cfg(std_os)]
        if provider::dump_cpu_count() > 1 {
            use super::vendor::Intel;

            let mut cores: Vec<CpuCore> = Vec::new();

            fn find_or_push(cores: &mut Vec<CpuCore>, core: CpuCore) {
                if let Some(c) = cores.iter_mut().find(|c| {
                    c.kind == core.kind && c.micro_arch == core.micro_arch && c.name == core.name
                }) {
                    c.count += core.count;
                    c.threads += core.threads;
                    if c.speed.is_none() && core.speed.is_some() {
                        c.speed = core.speed;
                    }
                } else {
                    cores.push(core);
                }
            }

            let dump_count = provider::dump_cpu_count();
            for cpu_idx in 0..dump_count {
                provider::set_dump_cpu(cpu_idx);

                let core_type = core_type_from_cpuid();
                let sig = CpuSignature::detect();
                let arch = CpuArch::find(&Cpu::raw_model_string(), sig, &vendor_str());
                let micro_arch = if is_intel() {
                    Intel::core_micro_arch(arch.micro_arch, core_type)
                } else {
                    arch.micro_arch
                };

                // Make sure we know the MicroArch before pushing to core types
                if micro_arch == MicroArch::Unknown {
                    continue;
                }

                let name_str = micro_arch.as_str();
                let name = if name_str != UNK {
                    Some(String::from(name_str))
                } else {
                    None
                };

                let cache = Cache::detect();
                let speed = Speed::detect_cpuid();
                let speed_opt = if speed.base > 0 { Some(speed) } else { None };

                find_or_push(
                    &mut cores,
                    CpuCore {
                        kind: core_type,
                        micro_arch,
                        name,
                        implementer: None,
                        cache,
                        speed: speed_opt,
                        count: 1,
                        threads: 1,
                    },
                );
            }

            for c in &mut cores {
                let smt = if c.kind == CoreType::Efficiency {
                    1
                } else {
                    cpuid_threads_per_core().max(1)
                };
                c.count = (c.threads / smt).max(1);
                if let Some(ref mut cache) = c.cache {
                    cache.resolve_share_counts(c.count, c.threads, 1);
                }
            }

            if cores.len() > 1 {
                return cores;
            }
        }

        Self::fallback_homogeneous(arch, topology)
    }

    /// Creates a single homogeneous CpuCore cluster fallback based on the package topology and architecture.
    pub fn fallback_homogeneous(arch: &CpuArch, topology: &Topology) -> Vec<CpuCore> {
        let speed = topology.speed;
        let speed_opt = if speed.base > 0 { Some(speed) } else { None };
        let cache = topology.cache;
        let sockets = topology.sockets.count.max(1);
        let cores_per_socket = (topology.cores.count / sockets).max(1);
        let threads_per_socket = (topology.threads.count / sockets).max(1);
        let name = if arch.code_name != UNK {
            Some(String::from(arch.code_name))
        } else {
            None
        };
        alloc::vec![CpuCore {
            kind: CoreType::Performance,
            micro_arch: arch.micro_arch,
            name,
            implementer: None,
            cache,
            speed: speed_opt,
            count: cores_per_socket,
            threads: threads_per_socket,
        }]
    }
}

#[cfg(std_os)]
impl Cpu {
    /// Detects CPU information from a `CpuDump` instance without touching any OS information.
    pub fn from_dump(dump: &provider::CpuDump) -> Self {
        provider::set_cpuid_provider(dump.clone());
        let cpu = Self::detect_cpuid();
        provider::reset_cpuid_provider();
        cpu
    }

    /// Detects CPU information from a CPUID dump file without touching any OS information.
    pub fn from_dump_file<P: AsRef<std::path::Path>>(path: P) -> Self {
        let dump = provider::CpuDump::parse_file(path);
        Self::from_dump(&dump)
    }

    /// Detects CPU information from a CPUID dump string without touching any OS information.
    pub fn from_dump_str(s: &str) -> Self {
        let dump = provider::CpuDump::parse_str(s);
        Self::from_dump(&dump)
    }
}

#[cfg(not(dos_os))]
impl Cpu {
    /// Enumerates all logical processors to discover unique core types.
    pub fn detect_core_types() -> Vec<CpuCore> {
        #[cfg(std_os)]
        if provider::info_source() == provider::CpuidInfoSource::DumpFile {
            let sig = CpuSignature::detect();
            let arch = CpuArch::find(&Self::raw_model_string(), sig, &vendor_str());
            let topo = Topology::detect_cpuid();
            return Self::detect_cpuid_core_types(&arch, &topo);
        }

        #[cfg(std_os)]
        {
            super::os::detect_live_core_types()
        }

        #[cfg(uefi)]
        {
            super::efi::detect_live_core_types()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::x86::get_feature_list;

    #[test]
    fn test_display_model_string_x32() {
        let dummy_sig = CpuSignature::new(0, 5, 0, 0, 0, DataSource::DefaultValue);

        // Test case for MicroArch::Am486
        let mut arch_am486 = CpuArch {
            micro_arch: MicroArch::Am486,
            code_name: "Am486DX2",
            brand_name: "AMD",
            vendor_string: String::from(VENDOR_AMD),
            ..Default::default()
        };

        let cpu_am486_dx2 = Cpu {
            extra: X86Data {
                arch: arch_am486.clone(),
                brand_id: 0,
                easter_egg: None,
                signature: dummy_sig,
                topology: Topology::default(),
                ..Default::default()
            },
            features: get_feature_list(),
            ..Default::default()
        };
        assert_eq!(cpu_am486_dx2.display_model_string(), "AMD 486 DX2");

        arch_am486.code_name = "Am486X2WB";
        let cpu_am486_x2wb = Cpu {
            extra: X86Data {
                arch: arch_am486.clone(),
                brand_id: 0,
                easter_egg: None,
                signature: dummy_sig,
                topology: Topology::default(),
                ..Default::default()
            },
            features: get_feature_list(),
            ..Default::default()
        };
        assert_eq!(
            cpu_am486_x2wb.display_model_string(),
            "AMD 486 DX2 with Write-Back Cache"
        );

        // Test case for MicroArch::I486
        let cpu_i486_dx = Cpu {
            extra: X86Data {
                arch: CpuArch {
                    micro_arch: MicroArch::I486,
                    code_name: "i80486DX",
                    brand_name: "Intel",
                    vendor_string: String::from(VENDOR_INTEL),
                    ..Default::default()
                },
                brand_id: 0,
                easter_egg: None,
                signature: dummy_sig,
                topology: Topology::default(),
                ..Default::default()
            },
            features: get_feature_list(),
            ..Default::default()
        };
        assert_eq!(cpu_i486_dx.display_model_string(), "Intel 486 DX");

        // Test case for "No CPUID"
        let cpu_no_cpuid = Cpu {
            extra: X86Data {
                arch: CpuArch {
                    vendor_string: String::from("UnknownVendor"),
                    ..CpuArch::default()
                },
                brand_id: 0,
                easter_egg: None,
                signature: CpuSignature::new(0, 6, 0, 0, 0, DataSource::DefaultValue),
                topology: Topology::default(),
                ..Default::default()
            },
            features: get_feature_list(),
            ..Default::default()
        };
        assert_eq!(cpu_no_cpuid.display_model_string(), UNK);
    }

    #[test]
    fn test_display_model_string() {
        // Test case for "Unknown"
        let cpu_unknown = Cpu {
            extra: X86Data {
                arch: CpuArch {
                    model: String::from("Unknown"),
                    vendor_string: String::from("UnknownVendor"),
                    ..CpuArch::default()
                },
                brand_id: 0,
                easter_egg: None,
                signature: CpuSignature::new(0, 6, 0, 0, 0, DataSource::DefaultValue),
                topology: Topology::default(),
                ..Default::default()
            },
            features: get_feature_list(),
            ..Default::default()
        };
        assert_eq!(cpu_unknown.display_model_string(), "Unknown");
    }
}
