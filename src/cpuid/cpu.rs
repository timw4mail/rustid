use crate::cpuid::brand::CpuBrand;
use crate::cpuid::micro_arch::{CpuArch, MicroArch};
use crate::cpuid::{fns, x86_cpuid};
use heapless::{String, Vec};

#[cfg(target_os = "none")]
use crate::println;
#[cfg(not(target_os = "none"))]
use std::println;

#[derive(Debug)]
pub struct CpuFeatures {
    list: Vec<&'static str, 64>,
}

impl ufmt::uDebug for CpuFeatures {
    fn fmt<W: ufmt::uWrite + ?Sized>(
        &self,
        f: &mut ufmt::Formatter<'_, W>,
    ) -> Result<(), W::Error> {
        f.write_str("CpuFeatures { list: [")?;
        for (i, feature) in self.list.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            f.write_str("\"")?;
            f.write_str(feature)?;
            f.write_str("\"")?;
        }
        f.write_str("] }")
    }
}

impl CpuFeatures {
    pub fn detect() -> Self {
        let mut out: Vec<_, _> = Vec::new();

        if fns::has_fpu() {
            let _ = out.push("FPU");
        };
        if fns::has_cx8() {
            let _ = out.push("CMPXCHG8B");
        };
        if fns::has_cmov() {
            let _ = out.push("CMOV");
        };
        if fns::has_3dnow() {
            let _ = out.push("3DNow!");
        };
        if fns::has_mmx() {
            let _ = out.push("MMX");
        };
        if fns::has_sse() {
            let _ = out.push("SSE");
        };
        if fns::has_amd64() {
            let _ = out.push("AMD64");
        };
        if fns::has_sse2() {
            let _ = out.push("SSSE2");
        };
        if fns::has_sse3() {
            let _ = out.push("SSE3");
        };
        if fns::has_sse41() {
            let _ = out.push("SSE4.1");
        };
        if fns::has_sse42() {
            let _ = out.push("SSE4.2");
        };
        if fns::has_ssse3() {
            let _ = out.push("SSSE3");
        };
        if fns::has_avx() {
            let _ = out.push("AVX");
        };
        if fns::has_avx2() {
            let _ = out.push("AVX2");
        };
        if fns::has_avx512f() {
            let _ = out.push("AVX512F");
        };
        if fns::has_fma() {
            let _ = out.push("FMA");
        };
        if fns::has_bmi1() {
            let _ = out.push("BMI1");
        };
        if fns::has_bmi2() {
            let _ = out.push("BMI2");
        };
        if fns::has_rdrand() {
            let _ = out.push("RDRAND");
        };

        Self {
            list: out,
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

impl ufmt::uDebug for CpuSignature {
    fn fmt<W: ufmt::uWrite + ?Sized>(
        &self,
        f: &mut ufmt::Formatter<'_, W>,
    ) -> Result<(), W::Error> {
        f.debug_struct("CpuSignature")?
            .field("extended_family", &self.extended_family)?
            .field("family", &self.family)?
            .field("extended_model", &self.extended_model)?
            .field("model", &self.model)?
            .field("stepping", &self.stepping)?
            .field("display_family", &self.display_family)?
            .field("display_model", &self.display_model)?
            .finish()
    }
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

impl ufmt::uDebug for Cpu {
    fn fmt<W: ufmt::uWrite + ?Sized>(
        &self,
        f: &mut ufmt::Formatter<'_, W>,
    ) -> Result<(), W::Error> {
        let mut none: String<64> = String::new();
        let _ = none.push_str("_None_");

        f.debug_struct("Cpu")?
            .field("arch", &self.arch)?
            .field("threads", &self.threads)?
            .field("signature", &self.signature)?
            .field("features", &self.features)?
            .finish()
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            arch: CpuArch::find(
                Self::model_string().as_str(),
                CpuSignature::detect(),
                CpuBrand::vendor_str(),
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

    fn display_model_string(&self) -> &str {
        if &self.arch.model != "Unknown" {
            return &self.arch.model;
        }

        match self.arch.micro_arch {
            // AMD
            MicroArch::Am486 => match self.arch.code_name {
                "Am486DX2" => "AMD 486 DX2",
                "Am486X2WB" => "AMD 486 DX2 with Write-Back Cache",
                "Am486DX4" => "AMD 486 DX4",
                "Am486DX4WB" => "AMD 486 DX4 with Write-Back Cache",
                _ => "486 Class CPU",
            },
            MicroArch::SSA5 | MicroArch::K5 => "AMD K5",

            //Intel
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
            MicroArch::P6PentiumII => "Intel Pentium II",
            MicroArch::P6PentiumIII => "Intel Pentium !!!",

            // Cyrix
            MicroArch::FiveX86 => "5x86",
            MicroArch::M1 => if fns::has_cx8() { "6x86L" } else { "6x86" },
            MicroArch::M2 => "6x86MX (MII)",

            _ => {
                if self.signature.family == 0
                    && self.signature.model == 0
                    && self.signature.extended_family == 0
                    && self.signature.extended_model == 0
                    && self.signature.stepping == 0
                {
                    "No CPUID, 486 or earlier CPU"
                } else {
                    "Unknown"
                }
            }
        }
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
        if !trimmed.is_empty() {
            let mut final_out: String<64> = String::new();
            let _ = final_out.push_str(trimmed);
            Some(final_out)
        } else {
            None
        }
    }

    pub fn debug(&self) {
        #[cfg(not(target_os = "none"))]
        println!("{:#?}", self);

        #[cfg(target_os = "none")]
        println!("{:?}", self);
    }

    // TODO: Show cpu process node
    // TODO: Show cpu cache size(s)
    // TODO: Show cpu speed
    pub fn display_table(&self) {
        println!();
        println!(
            "CPU Vendor:    {} ({})",
            self.arch.vendor_string.as_str(),
            self.arch.brand_name.as_str()
        );
        println!("CPU Name:      {}", self.display_model_string());
        println!("CPU Codename:  {}", self.arch.code_name);
        println!(
            "CPU Signature: Family {:X}h, Model {:X}h, Stepping {:X}h",
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
        self.features.list.iter().for_each(|feature| {
            println!("   {}", feature);
        });

        // if self.signature.display_family < 5 {
        //     println!("  FPU:      {}", self.features.fpu);
        // }
        //
        // if self.signature.display_family > 4 && self.signature.display_family <= 6 {
        //     println!("  CMPXCHG8B:{}", self.features.cx8);
        //     println!("  3DNow!:   {}", self.features.three_d_now);
        //     println!("  MMX:      {}", self.features.mmx);
        //     println!("  CMOV:     {}", self.features.cmov);
        //     println!("  SSE:      {}", self.features.sse);
        //     println!("  SSE2:     {}", self.features.sse2);
        //     println!("  AMD64:    {}", self.features.amd64);
        // }
        //
        // if self.features.amd64 {
        //     println!("  SSE3:     {}", self.features.sse3);
        //     println!("  SSSE3:    {}", self.features.ssse3);
        //     println!("  SSE4.1:   {}", self.features.sse41);
        //     println!("  SSE4.2:   {}", self.features.sse42);
        //     println!("  AVX:      {}", self.features.avx);
        //     println!("  AVX2:     {}", self.features.avx2);
        //     println!("  AVX-512F: {}", self.features.avx512f);
        //     println!("  FMA:      {}", self.features.fma);
        //     println!("  BMI1:     {}", self.features.bmi1);
        //     println!("  BMI2:     {}", self.features.bmi2);
        //     println!("  RDRAND:   {}", self.features.rdrand);
        // }

        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "none")]
    use crate::println;
    #[cfg(not(target_os = "none"))]
    use std::println;

    #[test]
    fn test_model_string() {
        let model = Cpu::model_string();
        println!("Model: {}", model);
        assert!(!model.is_empty());
    }
}
