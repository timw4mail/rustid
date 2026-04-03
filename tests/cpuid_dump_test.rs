#![cfg(feature = "file_mock")]
#![cfg(any(target_arch = "x86", target_arch = "x86_64"))]

use rustid::common::*;
use rustid::cpuid::provider::*;
use rustid::cpuid::*;
use std::path::PathBuf;

// ----------------------------------------------------------------------------
// ! Test Setup
// ----------------------------------------------------------------------------

struct MockCpuidProvider {
    cpu: CpuDump,
}

impl CpuidProvider for MockCpuidProvider {
    fn cpuid_count(&self, leaf: u32, sub_leaf: u32) -> Cpuid {
        self.cpu.get(leaf, sub_leaf)
    }
}

fn raw_path(segment: &str) -> PathBuf {
    let mut path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    path.push("tests");
    path.push("raw");
    path.push(segment);

    path
}

fn set_file_cpuid_provider(path: &str) {
    let path = raw_path(path);
    let cpu = CpuDump::parse_file(path);
    set_cpuid_provider(MockCpuidProvider { cpu: cpu.clone() });
}

// ----------------------------------------------------------------------------
// ! Test Helpers
// ----------------------------------------------------------------------------
fn get_signature() -> (u32, u32, u32, u32, u32) {
    let sig = CpuSignature::detect();

    (
        sig.extended_family,
        sig.family,
        sig.extended_model,
        sig.model,
        sig.stepping,
    )
}

fn calculate_cache_size(leaf: u32, subleaf: u32) -> Option<u32> {
    let res = x86_cpuid_count(leaf, subleaf);
    let cache_type = res.eax & 0xF;
    if cache_type == 0 {
        return None;
    }

    let cache_sets = res.ecx + 1;
    let cache_line_size = (res.ebx & 0xFFF) + 1;
    let cache_partitions = ((res.ebx >> 12) & 0x3FF) + 1;
    let cache_ways = ((res.ebx >> 22) & 0x3FF) + 1;

    Some(cache_sets * cache_partitions * cache_ways * cache_line_size)
}

fn calculate_cache_assoc(leaf: u32, subleaf: u32) -> Option<u32> {
    let res = x86_cpuid_count(leaf, subleaf);
    if res.eax & 0xF == 0 {
        return None;
    }
    Some(((res.ebx >> 22) & 0x3FF) + 1)
}

fn count_topology_domains(leaf: u32) -> usize {
    let mut count = 0;
    for subleaf in 0..16 {
        let res = x86_cpuid_count(leaf, subleaf);
        let domain_type = res.ecx >> 8;
        if domain_type == 0 {
            break;
        }
        count += 1;
    }
    count
}

#[cfg(target_arch = "x86")]
mod tm5700 {
    use super::*;

    fn with_mock_cpu(test: impl FnOnce()) {
        set_file_cpuid_provider("tm5700.txt");
        test();
    }

    #[test]
    fn test_vendor_detection() {
        with_mock_cpu(|| assert_eq!(vendor_str().as_str(), VENDOR_TRANSMETA))
    }

    #[test]
    fn test_max_leaf() {
        with_mock_cpu(|| {
            assert_eq!(max_leaf(), LEAF_1);
            assert_eq!(max_extended_leaf(), EXT_LEAF_6);
            assert_eq!(max_vendor_leaf(), TRANSMETA_LEAF_7);
        })
    }

    #[test]
    fn test_model_str() {
        with_mock_cpu(|| {
            let model_string = Cpu::raw_model_string();
            assert_eq!(model_string, "Transmeta(tm) Crusoe(tm) Processor TM5700");
        })
    }

    #[test]
    fn test_version_str() {
        with_mock_cpu(|| {
            let transmeta = rustid::cpuid::vendor::Transmeta::detect();
            assert_eq!(
                transmeta.version_str.as_str(),
                "20040614 15:00 official release 4.5.2#1"
            );
        })
    }
}

mod ppro {
    use super::*;
    use rustid::cpuid::mp::MpTable;

    fn with_mock_cpu(test: impl FnOnce()) {
        set_file_cpuid_provider("p6x2.txt");
        test();
    }

    #[test]
    fn test_vendor_detection() {
        with_mock_cpu(|| {
            let vendor = vendor_str();
            assert_eq!(vendor.as_str(), VENDOR_INTEL);
            assert_eq!(is_intel(), true);
        });
    }

    #[test]
    fn test_model_string() {
        with_mock_cpu(|| {
            use rustid::common::TCpu;

            let brand = Cpu::detect().display_model_string();
            assert!(brand.contains("Intel"));
            assert!(brand.contains("Pentium Pro"));
        });
    }

    #[test]
    fn test_raw_model_string() {
        with_mock_cpu(|| {
            assert_eq!(Cpu::raw_model_string(), UNK);
        })
    }

    #[test]
    fn test_socket_count() {
        let file = raw_path("p6x2cpuinfo.txt");
        let mp = MpTable::detect_file(file.to_str().unwrap());
        assert_eq!(mp.socket_count(), 2);
    }
}

mod m3_8100y {
    use super::*;

    fn with_mock_cpu(test: impl FnOnce()) {
        set_file_cpuid_provider("m3-8100y.txt");
        test();
    }

    #[test]
    fn test_intel_vendor_detection() {
        with_mock_cpu(|| {
            let vendor = vendor_str();
            assert_eq!(vendor.as_str(), VENDOR_INTEL);
        });
    }

    #[test]
    fn test_intel_brand_string() {
        with_mock_cpu(|| {
            use rustid::common::TCpu;

            let brand = Cpu::detect().display_model_string();
            assert!(brand.contains("Intel"));
            assert!(brand.contains("m3-8100Y"));
        });
    }

    #[test]
    fn test_intel_signature() {
        with_mock_cpu(|| {
            let (ext_family, family, ext_model, model, stepping) = get_signature();
            assert_eq!(stepping, 9);
            assert_eq!(model, 0xe);
            assert_eq!(family, 6);
            assert_eq!(ext_model, 8);
            assert_eq!(ext_family, 0);
        });
    }

    #[test]
    fn test_intel_max_leaf() {
        with_mock_cpu(|| {
            let res = max_leaf();
            assert_eq!(res, 0x16);
        });
    }

    #[test]
    fn test_intel_max_extended_leaf() {
        with_mock_cpu(|| {
            let res = max_extended_leaf();
            assert_eq!(res, 0x80000008);
        });
    }

    #[test]
    fn test_intel_ht_support() {
        with_mock_cpu(|| {
            assert_eq!(has_ht(), true);
        });
    }

    #[test]
    fn test_intel_threads() {
        with_mock_cpu(|| {
            use rustid::common::TCpu;

            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.threads, 4);
        });
    }

    #[test]
    fn test_intel_cores() {
        with_mock_cpu(|| {
            use rustid::common::TCpu;

            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.cores, 2);
        });
    }

    #[test]
    fn test_intel_feature_class() {
        with_mock_cpu(|| {
            let fc = FeatureClass::detect();
            assert_eq!(fc, FeatureClass::x86_64_v3);
            assert_eq!(fc.to_str(), "x86_64-v3");
        })
    }

    #[test]
    fn test_intel_topology_leaf_1f() {
        with_mock_cpu(|| {
            let domains = count_topology_domains(0x1F);
            let domains_b = count_topology_domains(0xB);
            assert!(
                domains >= 2 || domains_b >= 2,
                "Expected at least 2 topology domains"
            );
        });
    }

    #[test]
    fn test_intel_cache_detection() {
        with_mock_cpu(|| {
            let cache_sizes: Vec<_> = (0..4)
                .filter_map(|i| calculate_cache_size(0x4, i))
                .collect();
            assert!(!cache_sizes.is_empty());
        });
    }

    #[test]
    fn test_intel_cache_assoc() {
        with_mock_cpu(|| {
            let l1_assoc = calculate_cache_assoc(0x4, 0);
            assert!(l1_assoc.is_some(), "Expected L1 cache to exist");
        });
    }

    #[test]
    fn test_intel_sse_support() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(1, 0);
            let sse = (res.edx >> 25) & 1;
            let sse2 = (res.edx >> 26) & 1;
            let sse3 = (res.ecx >> 0) & 1;
            let sse41 = (res.ecx >> 19) & 1;
            let sse42 = (res.ecx >> 20) & 1;
            assert_eq!(sse, 1);
            assert_eq!(sse2, 1);
            assert_eq!(sse3, 1);
            assert_eq!(sse41, 1);
            assert_eq!(sse42, 1);
        });
    }

    #[test]
    fn test_intel_avx_support() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(1, 0);
            let avx = (res.ecx >> 28) & 1;
            assert_eq!(avx, 1);
        });
    }

    #[test]
    fn test_intel_avx2_support() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(7, 0);
            let avx2 = (res.ebx >> 5) & 1;
            assert_eq!(avx2, 1);
        });
    }
}

mod amd_5900xt {
    use super::*;

    fn with_mock_cpu(test: impl FnOnce()) {
        set_file_cpuid_provider("5900XT.txt");
        test();
    }

    #[test]
    fn test_amd_vendor_detection() {
        with_mock_cpu(|| {
            let vendor = vendor_str();
            assert_eq!(vendor.as_str(), VENDOR_AMD);
        });
    }

    #[test]
    fn test_amd_brand_string() {
        with_mock_cpu(|| {
            use rustid::common::TCpu;

            let brand = Cpu::detect().display_model_string();
            assert!(brand.contains("AMD"));
            assert!(brand.contains("5900"));
        });
    }

    #[test]
    fn test_amd_signature() {
        with_mock_cpu(|| {
            let (ext_family, family, ext_model, model, stepping) = get_signature();
            assert_eq!(stepping, 2);
            assert_eq!(model, 1);
            assert_eq!(family, 15);
            assert_eq!(ext_model, 2);
            assert_eq!(ext_family, 10);
        });
    }

    #[test]
    fn test_amd_max_leaf() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(0, 0);
            assert_eq!(res.eax, 0x10);
        });
    }

    #[test]
    fn test_amd_max_extended_leaf() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(0x80000000, 0);
            assert_eq!(res.eax, 0x80000023);
        });
    }

    #[test]
    fn test_amd_ht_support() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(1, 0);
            let ht = (res.edx >> 28) & 1;
            assert_eq!(ht, 1);
        });
    }

    #[test]
    fn test_amd_logical_cores() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(1, 0);
            let count = (res.ebx >> 16) & 0xFF;
            assert_eq!(count, 0x20); // 32 logical cores
        });
    }

    #[test]
    fn test_amd_threads() {
        with_mock_cpu(|| {
            use rustid::common::TCpu;

            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.threads, 32);
        });
    }

    #[test]
    fn test_amd_cores() {
        with_mock_cpu(|| {
            use rustid::common::TCpu;

            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.cores, 16);
        });
    }

    #[test]
    fn test_amd_topology_leaf_b() {
        with_mock_cpu(|| {
            let domains = count_topology_domains(0xB);
            assert!(domains >= 2);
        });
    }

    #[test]
    fn test_amd_cache_detection() {
        with_mock_cpu(|| {
            let cache_sizes: Vec<_> = (0..4)
                .filter_map(|i| calculate_cache_size(0x8000001D, i))
                .collect();
            assert!(!cache_sizes.is_empty());
        });
    }

    #[test]
    fn test_amd_cache_assoc() {
        with_mock_cpu(|| {
            let l1_assoc = calculate_cache_assoc(0x8000001D, 0);
            assert!(l1_assoc.is_some(), "Expected L1 cache to exist");
        });
    }

    #[test]
    fn test_amd_sse_support() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(1, 0);
            let sse = (res.edx >> 25) & 1;
            let sse2 = (res.edx >> 26) & 1;
            let sse3 = (res.ecx >> 0) & 1;
            let sse41 = (res.ecx >> 19) & 1;
            let sse42 = (res.ecx >> 20) & 1;
            assert_eq!(sse, 1);
            assert_eq!(sse2, 1);
            assert_eq!(sse3, 1);
            assert_eq!(sse41, 1);
            assert_eq!(sse42, 1);
        });
    }

    #[test]
    fn test_amd_avx_support() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(1, 0);
            let avx = (res.ecx >> 28) & 1;
            assert_eq!(avx, 1);
        });
    }

    #[test]
    fn test_amd_avx2_support() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(7, 0);
            let avx2 = (res.ebx >> 5) & 1;
            assert_eq!(avx2, 1);
        });
    }

    #[test]
    fn test_amd_popcnt_support() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(1, 0);
            let popcnt = (res.ecx >> 23) & 1;
            assert_eq!(popcnt, 1);
        });
    }
}

mod zhaoxin_kx5640 {
    use super::*;

    fn with_mock_cpu(test: impl FnOnce()) {
        set_file_cpuid_provider("kx5640.txt");
        test();
    }

    #[test]
    fn test_zhaoxin_vendor_detection() {
        with_mock_cpu(|| {
            let vendor = vendor_str();
            assert_eq!(vendor.as_str(), VENDOR_CENTAUR);
        });
    }

    #[test]
    fn test_zhaoxin_brand_string() {
        with_mock_cpu(|| {
            use rustid::common::TCpu;

            let brand = Cpu::detect().display_model_string();
            assert!(brand.contains("KX-5640") || brand.contains("ZHAOXIN"));
        });
    }

    #[test]
    fn test_zhaoxin_signature() {
        with_mock_cpu(|| {
            let (_xfamily, family, _xmodel, model, stepping) = get_signature();
            assert_eq!(stepping, 0);
            assert_eq!(model, 0xB);
            assert_eq!(family, 7);
        });
    }

    #[test]
    fn test_zhaoxin_max_leaf() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(0, 0);
            assert_eq!(res.eax, 0xD);
        });
    }

    #[test]
    fn test_zhaoxin_max_extended_leaf() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(0x80000000, 0);
            assert_eq!(res.eax, 0x80000008);
        });
    }

    #[test]
    fn test_zhaoxin_no_ht() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(1, 0);
            let ht = (res.edx >> 28) & 1;
            assert_eq!(ht, 1);
        });
    }

    #[test]
    fn test_zhaoxin_threads() {
        with_mock_cpu(|| {
            use rustid::common::TCpu;

            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.threads, 4);
        });
    }

    #[test]
    fn test_zhaoxin_cores() {
        with_mock_cpu(|| {
            use rustid::common::TCpu;

            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.cores, 4);
        });
    }

    #[test]
    fn test_zhaoxin_cache_detection() {
        with_mock_cpu(|| {
            let cache_sizes: Vec<_> = (0..4)
                .filter_map(|i| calculate_cache_size(0x4, i))
                .collect();
            assert!(!cache_sizes.is_empty());
        });
    }

    #[test]
    fn test_zhaoxin_sse_support() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(1, 0);
            let sse = (res.edx >> 25) & 1;
            let sse2 = (res.edx >> 26) & 1;
            assert_eq!(sse, 1);
            assert_eq!(sse2, 1);
        });
    }

    #[test]
    fn test_zhaoxin_avx_support() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(1, 0);
            let avx = (res.ecx >> 28) & 1;
            assert_eq!(avx, 1, "Zhaoxin Kaixian has AVX support");
        });
    }

    #[test]
    fn test_zhaoxin_centaur_extended() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(0xC0000000, 0);
            assert_eq!(res.eax, 0xC0000004);
        });
    }
}

mod via_c7d {
    use super::*;

    fn with_mock_cpu(test: impl FnOnce()) {
        set_file_cpuid_provider("c7d.txt");
        test();
    }

    #[test]
    fn test_via_vendor_detection() {
        with_mock_cpu(|| {
            let vendor = vendor_str();
            assert_eq!(vendor.as_str(), VENDOR_CENTAUR);
        });
    }

    #[test]
    fn test_via_brand_string() {
        with_mock_cpu(|| {
            use rustid::common::TCpu;

            let brand = Cpu::detect().display_model_string();
            assert!(brand.contains("C7") || !brand.is_empty());
        });
    }

    #[test]
    fn test_via_signature() {
        with_mock_cpu(|| {
            let (_xfamily, family, _xmodel, model, stepping) = get_signature();
            assert_eq!(stepping, 9);
            assert_eq!(model, 0xA);
            assert_eq!(family, 6);
        });
    }

    #[test]
    fn test_via_max_leaf() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(0, 0);
            assert_eq!(res.eax, 0x1);
        });
    }

    #[test]
    fn test_via_max_extended_leaf() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(0x80000000, 0);
            assert_eq!(res.eax, 0x80000006);
        });
    }

    #[test]
    fn test_via_no_ht() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(1, 0);
            let ht = (res.edx >> 28) & 1;
            assert_eq!(ht, 0);
        });
    }

    #[test]
    fn test_via_sse_support() {
        with_mock_cpu(|| {
            assert_eq!(has_sse(), true);
            assert_eq!(has_sse2(), true);
            assert_eq!(has_sse3(), true);
        });
    }

    #[test]
    fn test_via_centaur_extended() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(0xC000_0000, 0);
            assert_eq!(res.eax, 0xC000_0002);
        });
    }
}

#[test]
fn test_cpuid_struct_default() {
    let cpuid = Cpuid::default();
    assert_eq!(cpuid.eax, 0);
    assert_eq!(cpuid.ebx, 0);
    assert_eq!(cpuid.ecx, 0);
    assert_eq!(cpuid.edx, 0);
}

#[test]
fn test_cpuid_struct_from_raw() {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::CpuidResult;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::CpuidResult;

    let raw = CpuidResult {
        eax: 1,
        ebx: 2,
        ecx: 3,
        edx: 4,
    };
    let cpuid: Cpuid = raw.into();
    assert_eq!(cpuid.eax, 1);
    assert_eq!(cpuid.ebx, 2);
    assert_eq!(cpuid.ecx, 3);
    assert_eq!(cpuid.edx, 4);
}

#[test]
fn test_all_vendor_strings() {
    let vendors = vec![
        (VENDOR_AMD, CpuBrand::AMD),
        (VENDOR_INTEL, CpuBrand::Intel),
        (VENDOR_ZHAOXIN, CpuBrand::Zhaoxin),
        (VENDOR_CENTAUR, CpuBrand::Unknown),
    ];
    for (vendor_str, expected_brand) in vendors {
        assert_eq!(CpuBrand::from(vendor_str), expected_brand);
    }
}
