use crate::cpuid;
use crate::cpuid::fns;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuBrand {
    AMD,
    Intel,
    Unknown,
}

impl From<&String> for CpuBrand {
    fn from(brand: &String) -> Self {
        match brand.as_str() {
            "AuthenticAMD" => CpuBrand::AMD,
            "GenuineIntel" => CpuBrand::Intel,
            _ => CpuBrand::Unknown,
        }
    }
}

impl Into<String> for CpuBrand {
    fn into(self) -> String {
        match self {
            CpuBrand::AMD => "AMD".to_string(),
            CpuBrand::Intel => "Intel".to_string(),
            CpuBrand::Unknown => "Unknown".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct CpuFeatures {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// The Raw CPU Signature
pub struct RawSignature {
    extended_family: u32,
    family: u32,
    extended_model: u32,
    model: u32,
    stepping: u32,
}

impl RawSignature {
    pub fn new(ef: u32, f: u32, em: u32, m: u32, s: u32) -> Self {
        Self {
            extended_family: ef,
            family: f,
            extended_model: em,
            model: m,
            stepping: s,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// CPU Stepping, Model, and Family
pub struct DisplaySignature {
    family: u32,
    model: u32,
    stepping: u32,
}
impl From<RawSignature> for DisplaySignature {
    fn from(sig: RawSignature) -> Self {
        let display_family = if sig.family == 0xF {
            sig.family + sig.extended_family
        } else {
            sig.family
        };

        let display_model = if sig.family == 0x6 || sig.family == 0xF {
            (sig.extended_model << 4) + sig.model
        } else {
            sig.model
        };

        Self {
            family: display_family,
            model: display_model,
            stepping: sig.stepping,
        }
    }
}

#[derive(Debug)]
pub struct CpuSignature {
    raw: RawSignature,
    display: DisplaySignature,
}

impl CpuSignature {
    pub fn detect() -> Self {
        let res = cpuid::native_cpuid(1);
        let stepping = res.eax & 0xF;
        let model = (res.eax >> 4) & 0xF;
        let family = (res.eax >> 8) & 0xF;
        let extended_model = (res.eax >> 16) & 0xF;
        let extended_family = (res.eax >> 20) & 0xFF;

        let raw = RawSignature::new(extended_family, family, extended_model, model, stepping);

        Self {
            raw,
            display: DisplaySignature::from(raw),
        }
    }
}

#[derive(Debug)]
pub struct Cpu {
    brand: String,
    model: String,
    threads: u32,
    vendor_string: String,
    signature: CpuSignature,
    features: CpuFeatures,
}

impl Cpu {
    pub fn new() -> Self {
        let vendor_string = fns::vendor_id();

        Self {
            brand: CpuBrand::from(&vendor_string).into(),
            model: fns::model_string(),
            threads: fns::logical_cores(),
            vendor_string,
            signature: CpuSignature::detect(),
            features: CpuFeatures::detect(),
        }
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
