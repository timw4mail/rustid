use crate::cpuid;
use crate::cpuid::fns;
use crate::cpuid::micro_arch::CpuArch;

#[derive(Debug)]
pub struct CpuFeatures {
    fpu: bool,
    amd64: bool,
    three_d_now: bool,
    mmx: bool,
    sse: bool,
    sse2: bool,
    sse3: bool,
    sse41: bool,
    sse42: bool,
    ssse3: bool,
    avx: bool,
    avx2: bool,
    avx512f: bool,
    fma: bool,
    bmi1: bool,
    bmi2: bool,
}

impl CpuFeatures {
    pub fn detect() -> Self {
        Self {
            fpu: fns::has_fpu(),
            amd64: fns::has_amd64(),
            three_d_now: fns::has_3dnow(),
            mmx: fns::has_mmx(),
            sse: fns::has_sse(),
            sse2: fns::has_sse2(),
            sse3: fns::has_sse3(),
            sse41: fns::has_sse41(),
            sse42: fns::has_sse42(),
            ssse3: fns::has_ssse3(),
            avx: fns::has_avx(),
            avx2: fns::has_avx2(),
            avx512f: fns::has_avx512f(),
            fma: fns::has_fma(),
            bmi1: fns::has_bmi1(),
            bmi2: fns::has_bmi2(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CpuSignature {
    pub extended_family: u32,
    pub family: u32,
    pub extended_model: u32,
    pub model: u32,
    pub stepping: u32,
    pub display_family: u32,
    pub display_model: u32,
}

impl CpuSignature {
    pub fn detect() -> Self {
        let res = cpuid::native_cpuid(1);
        let stepping = res.eax & 0xF;
        let model = (res.eax >> 4) & 0xF;
        let family = (res.eax >> 8) & 0xF;
        let extended_model = (res.eax >> 16) & 0xF;
        let extended_family = (res.eax >> 20) & 0xFF;

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

        Self {
            extended_model,
            extended_family,
            family,
            model,
            stepping,
            display_family,
            display_model,
        }
    }
}

#[derive(Debug)]
pub struct Cpu {
    cpu_arch: CpuArch,
    easter_egg: Option<String>,
    threads: u32,
    signature: CpuSignature,
    features: CpuFeatures,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            cpu_arch: CpuArch::find(
                fns::model_string(),
                CpuSignature::detect(),
                fns::vendor_id().as_str(),
            ),
            easter_egg: fns::easter_egg(),
            threads: fns::logical_cores(),
            signature: CpuSignature::detect(),
            features: CpuFeatures::detect(),
        }
    }

    pub fn display(&self) {
        println!("{:#?}", self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_id() {
        let vendor = fns::vendor_id();
        println!("Vendor: {}", vendor);
        assert!(!vendor.is_empty());
    }

    #[test]
    fn test_model_string() {
        let model = fns::model_string();
        println!("Model: {}", model);
        assert!(!model.is_empty());
    }
}
