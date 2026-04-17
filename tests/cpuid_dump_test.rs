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
    path.push("cpuid");
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
        set_file_cpuid_provider("dump/tm5700.txt");
        test();
    }

    #[test]
    fn test_vendor_detection() {
        with_mock_cpu(|| assert_eq!(vendor_str(), VENDOR_TRANSMETA))
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
                transmeta.version_str,
                "20040614 15:00 official release 4.5.2#1"
            );
        })
    }

    #[test]
    fn test_threads() {
        with_mock_cpu(|| {
            use rustid::common::TCpu;
            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.threads, 1);
        });
    }

    #[test]
    fn test_cores() {
        with_mock_cpu(|| {
            use rustid::common::TCpu;
            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.cores, 1);
        });
    }
}

mod ppro {
    use super::*;
    use rustid::cpuid::mp::MpTable;

    fn with_mock_cpu(test: impl FnOnce()) {
        set_file_cpuid_provider("dump/p6x2.txt");
        test();
    }

    #[test]
    fn test_vendor_detection() {
        with_mock_cpu(|| {
            let vendor = vendor_str();
            assert_eq!(&*vendor, VENDOR_INTEL);
            assert!(is_intel());
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
        let file = raw_path("linux-cpuinfo/p6x2.txt");
        let mp = MpTable::detect_cpuinfo(file.to_str().unwrap());
        assert_eq!(mp.socket_count(), 2);
    }
}

mod m3_8100y {
    use super::*;

    fn with_mock_cpu(test: impl FnOnce()) {
        set_file_cpuid_provider("dump/m3-8100y.txt");
        test();
    }

    #[test]
    fn test_intel_vendor_detection() {
        with_mock_cpu(|| {
            let vendor = vendor_str();
            assert_eq!(&*vendor, VENDOR_INTEL);
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
            assert!(has_ht());
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
            use rustid::common::{TCpu, cache::CacheType};
            let cpu = Cpu::detect();
            let cache = cpu.topology.cache.expect("Expected cache to be detected");

            assert_eq!(cache.l1.size(), 65536, "L1 should be 64KB total");
            assert!(
                cache.l1.is_split(),
                "L1 cache should be split (separate I/D)"
            );

            if let Some(l2) = cache.l2 {
                assert_eq!(l2.kind(), CacheType::Unified);
                assert_eq!(l2.size(), 262144, "L2 should be 256KB");
                assert_eq!(l2.assoc(), 4, "L2 should be 4-way");
            }
        });
    }

    #[test]
    fn test_intel_cache_assoc() {
        with_mock_cpu(|| {
            use rustid::common::{TCpu, cache::Level1Cache};
            let cpu = Cpu::detect();
            let cache = cpu.topology.cache.expect("Expected cache to be detected");

            match cache.l1 {
                Level1Cache::Split { data, instruction } => {
                    assert_eq!(data.size(), 32768, "L1 data cache should be 32KB");
                    assert_eq!(data.assoc(), 8, "L1 data cache should be 8-way");
                    assert_eq!(
                        instruction.size(),
                        32768,
                        "L1 instruction cache should be 32KB"
                    );
                    assert_eq!(
                        instruction.assoc(),
                        8,
                        "L1 instruction cache should be 8-way"
                    );
                }
                _ => panic!("There's not unified cache here"),
            }
        });
    }

    #[test]
    fn test_intel_sse_support() {
        with_mock_cpu(|| {
            assert!(has_sse());
            assert!(has_sse2());
            assert!(has_sse3());
            assert!(has_sse41());
            assert!(has_sse42());
        });
    }

    #[test]
    fn test_intel_avx_support() {
        with_mock_cpu(|| {
            assert!(has_avx());
        });
    }

    #[test]
    fn test_intel_avx2_support() {
        with_mock_cpu(|| {
            assert!(has_avx2());
        });
    }

    #[test]
    fn test_intel_aes_support() {
        with_mock_cpu(|| {
            assert!(has_aes());
        });
    }

    #[test]
    fn test_intel_fpu_support() {
        with_mock_cpu(|| {
            assert!(has_fpu());
        });
    }

    #[test]
    fn test_intel_tsc_support() {
        with_mock_cpu(|| {
            assert!(has_tsc());
        });
    }

    #[test]
    fn test_intel_mmx_support() {
        with_mock_cpu(|| {
            assert!(has_mmx());
        });
    }

    #[test]
    fn test_intel_ssse3_support() {
        with_mock_cpu(|| {
            assert!(has_ssse3());
        });
    }

    #[test]
    fn test_intel_fma_support() {
        with_mock_cpu(|| {
            assert!(has_fma());
        });
    }

    #[test]
    fn test_intel_cx16_support() {
        with_mock_cpu(|| {
            assert!(has_cx16());
        });
    }

    #[test]
    fn test_intel_rdrand_support() {
        with_mock_cpu(|| {
            assert!(has_rdrand());
        });
    }

    #[test]
    fn test_intel_bmi1_support() {
        with_mock_cpu(|| {
            assert!(has_bmi1());
        });
    }

    #[test]
    fn test_intel_bmi2_support() {
        with_mock_cpu(|| {
            assert!(has_bmi2());
        });
    }

    #[test]
    fn test_intel_f16c_support() {
        with_mock_cpu(|| {
            assert!(has_f16c());
        });
    }
}

mod amd_7950x3d {
    use super::*;

    #[test]
    fn test_topology() {
        set_file_cpuid_provider("dump/7950x3d.txt");
        use rustid::common::TCpu;
        let cpu = Cpu::detect();

        assert_eq!(cpu.topology.dies, 2);
        assert_eq!(cpu.topology.threads, 32);
        assert_eq!(cpu.topology.cores, 16);
        assert_eq!(cpu.topology.sockets, 1);
    }
}

mod amd_5900xt {
    use super::*;

    fn with_mock_cpu(test: impl FnOnce()) {
        set_file_cpuid_provider("dump/5900XT.txt");
        test();
    }

    #[test]
    fn test_amd_vendor_detection() {
        with_mock_cpu(|| {
            let vendor = vendor_str();
            assert_eq!(&*vendor, VENDOR_AMD);
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
            assert_eq!(max_leaf(), 0x10);
        });
    }

    #[test]
    fn test_amd_max_extended_leaf() {
        with_mock_cpu(|| {
            assert_eq!(max_extended_leaf(), 0x80000023);
        });
    }

    #[test]
    fn test_amd_ht_support() {
        with_mock_cpu(|| {
            assert!(has_ht());
        });
    }

    #[test]
    fn test_amd_logical_cores() {
        with_mock_cpu(|| {
            use rustid::common::TCpu;
            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.threads, 32);
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
            use rustid::common::{TCpu, cache::CacheType};
            let cpu = Cpu::detect();
            let cache = cpu.topology.cache.expect("Expected cache to be detected");

            assert_eq!(
                cache.l1.size(),
                65536,
                "L1 cache should be 64KB total (32KB data + 32KB instruction)"
            );

            if let Some(l2) = cache.l2 {
                assert_eq!(l2.kind(), CacheType::Unified);
                assert_eq!(l2.share_count(), 2);
                assert_eq!(l2.size(), 524288, "L2 should be 512KB");
                assert_eq!(l2.assoc(), 8, "L2 should be 8-way");
            }

            if let Some(l3) = cache.l3 {
                assert_eq!(l3.kind(), CacheType::Unified);
                assert_eq!(l3.share_count(), 16);
                assert_eq!(l3.size(), 33554432, "L3 should be 32MB");
                assert_eq!(l3.assoc(), 16, "L3 should be 16-way");
            }
        });
    }

    #[test]
    fn test_amd_cache_assoc() {
        with_mock_cpu(|| {
            use rustid::common::{TCpu, cache::Level1Cache};
            let cpu = Cpu::detect();
            let cache = cpu.topology.cache.expect("Expected cache to be detected");

            match cache.l1 {
                Level1Cache::Split { data, instruction } => {
                    assert_eq!(data.size(), 32768, "L1 data should be 32KB");
                    assert_eq!(data.assoc(), 8, "L1 data should be 8-way");
                    assert_eq!(instruction.size(), 32768, "L1 instruction should be 32KB");
                    assert_eq!(instruction.assoc(), 8, "L1 instruction should be 8-way");
                }
                _ => panic!("There's not unified cache here"),
            }
        });
    }

    #[test]
    fn test_amd_sse_support() {
        with_mock_cpu(|| {
            assert!(has_sse());
            assert!(has_sse2());
            assert!(has_sse3());
            assert!(has_sse41());
            assert!(has_sse42());
        });
    }

    #[test]
    fn test_amd_avx_support() {
        with_mock_cpu(|| {
            assert!(has_avx());
        });
    }

    #[test]
    fn test_amd_avx2_support() {
        with_mock_cpu(|| {
            assert!(has_avx2());
        });
    }

    #[test]
    fn test_amd_popcnt_support() {
        with_mock_cpu(|| {
            assert!(has_popcnt());
        });
    }

    #[test]
    fn test_amd_aes_support() {
        with_mock_cpu(|| {
            assert!(has_aes());
        });
    }

    #[test]
    fn test_amd_mmx_support() {
        with_mock_cpu(|| {
            assert!(has_mmx());
        });
    }

    #[test]
    fn test_amd_ssse3_support() {
        with_mock_cpu(|| {
            assert!(has_ssse3());
        });
    }

    #[test]
    fn test_amd_fma_support() {
        with_mock_cpu(|| {
            assert!(has_fma());
        });
    }

    #[test]
    fn test_amd_sse4a_support() {
        with_mock_cpu(|| {
            assert!(has_sse4a());
        });
    }

    #[test]
    fn test_amd_amd64_support() {
        with_mock_cpu(|| {
            assert!(has_amd64());
        });
    }

    #[test]
    fn test_amd_f16c_support() {
        with_mock_cpu(|| {
            assert!(has_f16c());
        });
    }

    #[test]
    fn test_amd_x2apic_support() {
        with_mock_cpu(|| {
            assert!(has_x2apic());
        });
    }

    #[test]
    fn test_amd_3dnow_support() {
        with_mock_cpu(|| {
            assert!(!has_3dnow());
            assert!(!has_3dnow_plus());
        });
    }
}

mod zhaoxin_kx5640 {
    use super::*;

    fn with_mock_cpu(test: impl FnOnce()) {
        set_file_cpuid_provider("dump/kx5640.txt");
        test();
    }

    #[test]
    fn test_zhaoxin_vendor_detection() {
        with_mock_cpu(|| {
            let vendor = vendor_str();
            assert_eq!(&*vendor, VENDOR_CENTAUR);
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
            assert!(has_ht());
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
            use rustid::common::{Level1Cache, TCpu, cache::CacheType};
            let cpu = Cpu::detect();
            let cache = cpu.topology.cache.expect("Expected cache to be detected");

            assert!(cache.l1.size() > 0, "L1 cache should exist");

            match cache.l1 {
                Level1Cache::Split { data, instruction } => {
                    assert_eq!(data.size(), 32768, "L1 data should be 32KB");
                    assert_eq!(data.assoc(), 8, "L1 data should have associativity");
                    assert_eq!(instruction.size(), 32768);
                    assert_eq!(instruction.assoc(), 8);
                }
                _ => panic!("There's not unified cache here"),
            }

            if let Some(l2) = cache.l2 {
                assert_eq!(l2.kind(), CacheType::Unified);
                assert_eq!(l2.size(), 4194304);
            }
        });
    }

    #[test]
    fn test_zhaoxin_sse_support() {
        with_mock_cpu(|| {
            assert!(has_sse());
            assert!(has_sse2());
        });
    }

    #[test]
    fn test_zhaoxin_avx_support() {
        with_mock_cpu(|| {
            assert!(has_avx());
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
        set_file_cpuid_provider("dump/c7d.txt");
        test();
    }

    #[test]
    fn test_via_vendor_detection() {
        with_mock_cpu(|| {
            let vendor = vendor_str();
            assert_eq!(&*vendor, VENDOR_CENTAUR);
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
            assert_eq!(max_leaf(), 0x1);
        });
    }

    #[test]
    fn test_via_max_extended_leaf() {
        with_mock_cpu(|| {
            assert_eq!(max_extended_leaf(), 0x80000006);
        });
    }

    #[test]
    fn test_via_no_ht() {
        with_mock_cpu(|| {
            assert!(!has_ht());
        });
    }

    #[test]
    fn test_via_cache_detection() {
        with_mock_cpu(|| {
            use rustid::common::{TCpu, cache::CacheType};
            let cpu = Cpu::detect();
            let cache = cpu.topology.cache.expect("Expected cache to be detected");

            assert!(cache.l1.size() > 0, "L1 cache should exist");

            if let Some(l2) = cache.l2 {
                assert_eq!(l2.kind(), CacheType::Unified);
                assert!(l2.size() > 0);
            }
        });
    }

    #[test]
    fn test_via_sse_support() {
        with_mock_cpu(|| {
            assert!(has_sse());
            assert!(has_sse2());
            assert!(has_sse3());
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

#[cfg(target_arch = "x86")]
mod vortex86dx3 {
    use rustid::cpuid::{has_ht, has_mmx, max_extended_leaf};

    use super::*;

    fn with_mock_cpu(test: impl FnOnce()) {
        set_file_cpuid_provider("dump/vortex86dx3.txt");
        test();
    }

    #[test]
    fn test_vortex86_vendor_detection() {
        with_mock_cpu(|| {
            let vendor = vendor_str();
            assert_eq!(&*vendor, VENDOR_DMP);
        });
    }

    #[test]
    fn test_vortex86_brand_string() {
        with_mock_cpu(|| {
            use rustid::common::TCpu;

            let brand = Cpu::detect().display_model_string();
            assert!(brand.contains("Vortex86"));
        });
    }

    #[test]
    fn test_vortex86_signature() {
        with_mock_cpu(|| {
            let (_xfamily, family, _xmodel, model, stepping) = get_signature();
            assert_eq!(stepping, 1);
            assert_eq!(model, 1);
            assert_eq!(family, 6);
        });
    }

    #[test]
    fn test_vortex86_max_leaf() {
        with_mock_cpu(|| {
            let res = max_leaf();
            assert_eq!(res, 0x3);
        });
    }

    #[test]
    fn test_vortex86_max_extended_leaf() {
        with_mock_cpu(|| {
            let res = max_extended_leaf();
            assert_eq!(res, 0x80000004);
        });
    }

    #[test]
    fn test_vortex86_no_ht() {
        with_mock_cpu(|| {
            let res = has_ht();
            assert_eq!(res, false);
        });
    }

    #[test]
    fn test_vortex86_threads() {
        with_mock_cpu(|| {
            use rustid::common::TCpu;
            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.threads, 1);
        });
    }

    #[test]
    fn test_vortex86_cores() {
        with_mock_cpu(|| {
            use rustid::common::TCpu;
            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.cores, 1);
        });
    }

    #[test]
    fn test_vortex86_cache_detection() {
        with_mock_cpu(|| {
            use rustid::common::{Level1Cache, TCpu, cache::CacheType};
            let cpu = Cpu::detect();
            let cache = cpu.topology.cache.expect("Expected cache to be detected");

            assert_eq!(
                cache.l1.size(),
                32768,
                "L1 should be 32KB (16KB data + 16KB instruction)"
            );
            assert!(cache.l1.is_split(), "L1 should be split");

            match cache.l1 {
                Level1Cache::Unified(_) => panic!("Expected split L1 cache"),
                Level1Cache::Split { data, instruction } => {
                    assert_eq!(data.size(), 16384, "L1 data should be 16KB");
                    assert_eq!(data.assoc(), 4, "L1 data should be 4-way");
                    assert_eq!(instruction.size(), 16384, "L1 instruction should be 16KB");
                    assert_eq!(instruction.assoc(), 4, "L1 instruction should be 4-way");
                }
            }

            if let Some(l2) = cache.l2 {
                assert_eq!(l2.kind(), CacheType::Unified);
                assert_eq!(l2.size(), 262144, "L2 should be 256KB");
                assert_eq!(l2.assoc(), 4, "L2 should be 4-way");
            }
        });
    }

    #[test]
    fn test_vortex86_has_mmx() {
        with_mock_cpu(|| {
            let res = has_mmx();
            assert_eq!(res, true, "Vortex86 has MMX support");
        });
    }

    #[test]
    fn test_vortex86_feature_class() {
        with_mock_cpu(|| {
            let fc = FeatureClass::detect();
            assert!(matches!(fc, FeatureClass::i686_SSE));
        })
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
