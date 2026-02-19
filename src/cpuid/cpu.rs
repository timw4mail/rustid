use crate::cpuid::brand::CpuBrand;
use crate::cpuid::micro_arch::{CpuArch, MicroArch};
use crate::cpuid::{fns, x86_cpuid};
use heapless::String;

#[cfg(target_os = "none")]
use crate::println;
#[cfg(not(target_os = "none"))]
use std::println;

#[derive(Debug)]
pub struct CpuFeatures {
    cx8: bool,
    cmov: bool,
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
    rdrand: bool,
}

impl CpuFeatures {
    pub fn detect() -> Self {
        Self {
            cx8: fns::has_cx8(),
            cmov: fns::has_cmov(),
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
            rdrand: fns::has_rdrand(),
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
        let res = x86_cpuid(1);
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
    pub arch: CpuArch,
    pub easter_egg: Option<String<64>>,
    pub threads: u32,
    pub signature: CpuSignature,
    pub features: CpuFeatures,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            arch: CpuArch::find(
                Self::model_string().as_str(),
                CpuSignature::detect(),
                CpuBrand::vendor_id().as_str(),
            ),
            easter_egg: Self::easter_egg(),
            threads: fns::logical_cores(),
            signature: CpuSignature::detect(),
            features: CpuFeatures::detect(),
        }
    }

    /// Gets the CPU model string.
    fn model_string() -> String<64> {
        let mut model: String<64> = String::new();
        if fns::max_extended_leaf() < 0x8000_0004 {
            let _ = model.push_str("Unknown");
            return model;
        }

        for leaf in 0x8000_0002..=0x8000_0004 {
            let res = x86_cpuid(leaf);
            for reg in &[res.eax, res.ebx, res.ecx, res.edx] {
                for &b in &reg.to_le_bytes() {
                    if b != 0 {
                        let _ = model.push(b as char);
                    }
                }
            }
        }

        let trimmed = model.trim();
        let mut out: String<64> = String::new();
        let _ = out.push_str(trimmed);
        out
    }

    fn display_model_string(&self) -> String<64> {
        if &self.arch.model != "Unknown" {
            return self.arch.model.clone();
        }

        let mut out: String<64> = String::new();

        let str = match self.arch.micro_arch {
            MicroArch::Am486 => match self.arch.code_name {
                "Am486DX2" => "AMD 486 DX2",
                "Am486X2WB" => "AMD 486 DX2 with Write-Back Cache",
                "Am486DX4" => "AMD 486 DX4",
                "Am486DX4WB" => "AMD 486 DX4 with Write-Back Cache",
                _ => "486 Class CPU",
            },
            MicroArch::I486 => match self.arch.code_name {
                "i80486DX" => "Intel or AMD 486 DX",
                "i80486DX-50" => "Intel or AMD 486 DX-50",
                "i80486SX" => "Intel or AMD 486 SX",
                "i80486DX2" => "Intel 486 DX2",
                "i80486SL" => "Intel 486 SL",
                "i80486SX2" => "Intel or AMD 486 SX2",
                "i80486DX2WB" => "Intel 486 DX2 with Write-Back Cache",
                "i80486DX4" => "Intel 486 DX4",
                "i80486DX4WB" => "Intel 486 DX4 with Write-Back Cache",
                _ => "486 Class CPU",
            },
            MicroArch::P6Pro => "Intel Pentium Pro",
            MicroArch::SSA5 | MicroArch::K5 => "AMD K5",
            _ => "Unknown",
        };

        let _ = out.push_str(str.trim());

        out
    }

    fn easter_egg() -> Option<String<64>> {
        let mut out: String<64> = String::new();

        let addr = match CpuBrand::detect() {
            CpuBrand::AMD => 0x8FFF_FFFF,
            CpuBrand::Rise => 0x0000_5A4E,
            _ => 1,
        };

        if addr != 1 {
            let res = x86_cpuid(addr);

            for &reg in &[res.eax, res.ebx, res.ecx, res.edx] {
                let bytes = reg.to_le_bytes();
                for &b in &bytes {
                    if b != 0 {
                        let _ = out.push(b as char);
                    }
                }
            }
        }

        let trimmed = out.trim();
        if trimmed.len() > 0 {
            let mut final_out: String<64> = String::new();
            let _ = final_out.push_str(trimmed);
            Some(final_out)
        } else {
            None
        }
    }

    #[cfg(not(target_os = "none"))]
    pub fn debug(&self) {
        println!("{:#?}", self);
    }

    pub fn display_table(&self) {
        println!();

        println!(
            "CPU Vendor:    {} ({})",
            self.arch.vendor_string.as_str(),
            self.arch.brand_name.as_str()
        );
        println!("CPU Name:      {}", self.display_model_string().as_str());
        println!("CPU Codename:  {}", self.arch.code_name);
        println!(
            "CPU Signature: Family {}, Model {}, Stepping {}",
            self.signature.display_family, self.signature.display_model, self.signature.stepping
        );
        println!(
            "               ({}, {}, {}, {}, {})",
            self.signature.extended_family,
            self.signature.family,
            self.signature.extended_model,
            self.signature.model,
            self.signature.stepping
        );

        if self.threads > 1 {
            println!("Logical Cores: {}", self.threads);
        }

        if let Some(easter_egg) = &self.easter_egg {
            println!("Easter Egg: {}", easter_egg.as_str());
        }

        println!("Features:");
        if self.signature.display_family < 5 {
            println!("  FPU:      {}", self.features.fpu);
        }

        if self.signature.display_family > 4 && self.signature.display_family <= 6 {
            println!("  CMPXCHG8B:{}", self.features.cx8);
            println!("  3DNow!:   {}", self.features.three_d_now);
            println!("  MMX:      {}", self.features.mmx);
            println!("  CMOV:     {}", self.features.cmov);
            println!("  SSE:      {}", self.features.sse);
            println!("  SSE2:     {}", self.features.sse2);
            println!("  AMD64:    {}", self.features.amd64);
        }

        if self.features.amd64 {
            println!("  SSE3:     {}", self.features.sse3);
            println!("  SSSE3:    {}", self.features.ssse3);
            println!("  SSE4.1:   {}", self.features.sse41);
            println!("  SSE4.2:   {}", self.features.sse42);
            println!("  AVX:      {}", self.features.avx);
            println!("  AVX2:     {}", self.features.avx2);
            println!("  AVX-512F: {}", self.features.avx512f);
            println!("  FMA:      {}", self.features.fma);
            println!("  BMI1:     {}", self.features.bmi1);
            println!("  BMI2:     {}", self.features.bmi2);
            println!("  RDRAND:   {}", self.features.rdrand);
        }

        println!();
    }
}

#[cfg(test)]
#[cfg(not(target_os = "none"))]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_id() {
        let vendor = CpuBrand::vendor_id();
        println!("Vendor: {}", vendor);
        assert!(!vendor.is_empty());
    }

    #[test]
    fn test_model_string() {
        let model = Cpu::model_string();
        println!("Model: {}", model);
        assert!(!model.is_empty());
    }
}
