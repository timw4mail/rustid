use crate::cpuid::brand::CpuBrand;
use crate::cpuid::micro_arch::CpuArch;
use crate::cpuid::{fns, x86_cpuid};
use heapless::String;

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
    rdrand: bool,
}

impl ufmt::uDebug for CpuFeatures {
    fn fmt<W: ufmt::uWrite + ?Sized>(
        &self,
        f: &mut ufmt::Formatter<'_, W>,
    ) -> Result<(), W::Error> {
        let mut s = f.debug_struct("CpuFeatures")?;
        s.field("fpu", &self.fpu)?
            .field("amd64", &self.amd64)?
            .field("three_d_now", &self.three_d_now)?
            .field("mmx", &self.mmx)?
            .field("sse", &self.sse)?
            .field("sse2", &self.sse2)?
            .field("sse3", &self.sse3)?
            .field("sse41", &self.sse41)?
            .field("sse42", &self.sse42)?
            .field("ssse3", &self.ssse3)?
            .field("avx", &self.avx)?
            .field("avx2", &self.avx2)?
            .field("avx512f", &self.avx512f)?
            .field("fma", &self.fma)?
            .field("bmi1", &self.bmi1)?
            .field("bmi2", &self.bmi2)?
            .field("rdrand", &self.rdrand)?
            .finish()
    }
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
    pub cpu_arch: CpuArch,
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
        f.write_str("Cpu { cpu_arch: ")?;
        ufmt::uDebug::fmt(&self.cpu_arch, f)?;
        f.write_str(", easter_egg: ")?;
        match &self.easter_egg {
            Some(s) => {
                f.write_str("Some(\"")?;
                f.write_str(s.as_str())?;
                f.write_str("\")")?;
            }
            None => f.write_str("None")?,
        }
        f.write_str(", threads: ")?;
        ufmt::uDebug::fmt(&self.threads, f)?;
        f.write_str(", signature: ")?;
        ufmt::uDebug::fmt(&self.signature, f)?;
        f.write_str(", features: ")?;
        ufmt::uDebug::fmt(&self.features, f)?;
        f.write_str(" }")
    }
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            cpu_arch: CpuArch::find(
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
        // Check if extended functions are supported
        let max_extended_leaf = x86_cpuid(0x8000_0000).eax;
        if max_extended_leaf < 0x8000_0004 {
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

    pub fn display(&self) {
        #[cfg(not(target_os = "none"))]
        std::println!("{:#?}", self);

        #[cfg(target_os = "none")]
        {
            use crate::dos::DosWriter;
            use ufmt::uWrite;

            let mut writer = DosWriter;
            let _ = ufmt::uwriteln!(writer, "CPU INFO:\n");
            let mut writer = DosWriter;
            let _ = ufmt::uwriteln!(writer, "{:#?}", self);
        }
    }

    pub fn display_table(&self) {
        #[cfg(not(target_os = "none"))]
        use std::println;

        #[cfg(target_os = "none")]
        use crate::println;

        println!("CPU Name:     {}", self.cpu_arch.model);
        println!("CPU Vendor:    {}", self.cpu_arch.vendor_string);
        println!(
            "CPU Signature: Family {}, Model {}, Stepping {}",
            self.signature.display_family, self.signature.display_model, self.signature.stepping
        );
        println!(
            "          Raw: EF {}, F {}, EM {}, M {}, S {}",
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
            println!("Easter Egg: {}", easter_egg);
        }

        println!("Features:");
        println!("  FPU:      {}", self.features.fpu);
        println!("  AMD64:    {}", self.features.amd64);
        println!("  3DNow!:   {}", self.features.three_d_now);
        println!("  MMX:      {}", self.features.mmx);
        println!("  SSE:      {}", self.features.sse);
        println!("  SSE2:     {}", self.features.sse2);
        println!("  SSE3:     {}", self.features.sse3);
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
}

#[cfg(test)]
#[cfg(not(target_os = "none"))]
mod tests {
    use super::*;

    #[test]
    fn test_vendor_id() {
        let vendor = CpuBrand::vendor_id();
        std::println!("Vendor: {}", vendor);
        assert!(!vendor.is_empty());
    }

    #[test]
    fn test_model_string() {
        let model = Cpu::model_string();
        std::println!("Model: {}", model);
        assert!(!model.is_empty());
    }
}
