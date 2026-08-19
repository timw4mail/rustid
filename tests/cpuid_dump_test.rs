#![cfg(x86_cpu)]

use rustid::common::TDetect;
use rustid::common::*;
use rustid::x86::provider::*;
use rustid::x86::*;
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

    fn info_source(&self) -> CpuidInfoSource {
        CpuidInfoSource::DumpFile
    }
}

fn raw_path(segment: &str) -> PathBuf {
    let mut path =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("Couldn't find repo dir"));
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

fn assert_cache_counts(
    cpu: &Cpu,
    l1d: (u32, &str),
    l1i: (u32, &str),
    l2: Option<(u32, &str)>,
    l3: Option<(u32, &str)>,
) {
    use rustid::common::cache::Level1Cache;

    let cache = cpu.topology.cache.expect("Expected cache to be detected");

    match cache.l1 {
        Level1Cache::Split { data, instruction } => {
            let (expected_d_inst, expected_d_prefix) = l1d;
            let (expected_i_inst, expected_i_prefix) = l1i;

            let d_inst = CpuDisplay::x86_cache_instances(
                data.share_count(),
                cpu.topology.cores.count,
                cpu.topology.threads.count,
                cpu.topology.sockets.count,
            );
            let i_inst = CpuDisplay::x86_cache_instances(
                instruction.share_count(),
                cpu.topology.cores.count,
                cpu.topology.threads.count,
                cpu.topology.sockets.count,
            );
            assert_eq!(d_inst, expected_d_inst, "L1d instance count mismatch");
            assert_eq!(i_inst, expected_i_inst, "L1i instance count mismatch");

            let d_prefix = CpuDisplay::x86_cache_count(
                data.share_count(),
                cpu.topology.cores.count,
                cpu.topology.threads.count,
                cpu.topology.sockets.count,
            );
            let i_prefix = CpuDisplay::x86_cache_count(
                instruction.share_count(),
                cpu.topology.cores.count,
                cpu.topology.threads.count,
                cpu.topology.sockets.count,
            );
            assert_eq!(d_prefix, expected_d_prefix, "L1d prefix mismatch");
            assert_eq!(i_prefix, expected_i_prefix, "L1i prefix mismatch");
        }
        _ => panic!("Expected split L1 cache"),
    }

    if let Some((expected_l2_inst, expected_l2_prefix)) = l2 {
        let l2 = cache.l2.expect("Expected L2 cache to be present");
        let l2_inst = CpuDisplay::x86_cache_instances(
            l2.share_count(),
            cpu.topology.cores.count,
            cpu.topology.threads.count,
            cpu.topology.sockets.count,
        );
        assert_eq!(l2_inst, expected_l2_inst, "L2 instance count mismatch");

        let l2_prefix = CpuDisplay::x86_cache_count(
            l2.share_count(),
            cpu.topology.cores.count,
            cpu.topology.threads.count,
            cpu.topology.sockets.count,
        );
        assert_eq!(l2_prefix, expected_l2_prefix, "L2 prefix mismatch");
    }

    if let Some((expected_l3_inst, expected_l3_prefix)) = l3 {
        let l3 = cache.l3.expect("Expected L3 cache to be present");
        let l3_inst = CpuDisplay::x86_cache_instances(
            l3.share_count(),
            cpu.topology.cores.count,
            cpu.topology.threads.count,
            cpu.topology.sockets.count,
        );
        assert_eq!(l3_inst, expected_l3_inst, "L3 instance count mismatch");

        let l3_prefix = CpuDisplay::x86_cache_count(
            l3.share_count(),
            cpu.topology.cores.count,
            cpu.topology.threads.count,
            cpu.topology.sockets.count,
        );
        assert_eq!(l3_prefix, expected_l3_prefix, "L3 prefix mismatch");
    }
}

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
    fn test_signature() {
        with_mock_cpu(|| {
            assert_eq!(get_signature(), (0, 5, 0, 4, 3));
        });
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
            let transmeta = rustid::x86::vendor::Transmeta::detect();
            assert_eq!(
                transmeta.version_str,
                "20040614 15:00 official release 4.5.2#1"
            );
        })
    }

    #[test]
    fn test_topology() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.threads.count, 1);
            assert_eq!(cpu.topology.cores.count, 1);
            assert_eq!(cpu.topology.sockets.count, 1);
        });
    }

    #[test]
    fn test_cache_detection() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            let cache = cpu.topology.cache.expect("Expected cache to be detected");
            assert_eq!(cache.l1.size(), 131072, "TM5700 L1 should be 128KB");
            assert_eq!(
                cache.l2.expect("L2").size(),
                262144,
                "TM5700 L2 should be 256KB"
            );
        });
    }
}

mod ppro {
    use super::*;

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
    fn test_signature() {
        with_mock_cpu(|| {
            assert_eq!(get_signature(), (0, 6, 0, 1, 9));
        });
    }

    #[test]
    fn test_model_string() {
        with_mock_cpu(|| {
            let brand = Cpu::detect().display_model_string();
            assert_eq!(brand, "Intel Pentium Pro");
        });
    }

    #[test]
    fn test_raw_model_string() {
        with_mock_cpu(|| {
            assert_eq!(Cpu::raw_model_string(), UNK);
        })
    }

    #[test]
    fn test_topology() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.sockets.count, 1);
            assert_eq!(cpu.topology.cores.count, 1);
            assert_eq!(cpu.topology.threads.count, 1);
        });
    }

    #[test]
    fn test_cache_detection() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            let cache = cpu.topology.cache.expect("Expected cache to be detected");
            assert_eq!(cache.l1.size(), 16384, "Pentium Pro L1 should be 16KB");
            assert_eq!(
                cache.l2.expect("L2").size(),
                1048576,
                "Pentium Pro L2 should be 1MB"
            );
        });
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
            let brand = Cpu::detect().display_model_string();
            assert!(brand.contains("Intel"));
            assert!(brand.contains("m3-8100Y"));
        });
    }

    #[test]
    fn test_intel_signature() {
        with_mock_cpu(|| {
            assert_eq!(get_signature(), (0, 6, 8, 14, 9));
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
    fn test_intel_topology() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.sockets.count, 1);
            assert_eq!(cpu.topology.cores.count, 2);
            assert_eq!(cpu.topology.threads.count, 4);
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
            use rustid::common::cache::CacheType;
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
            use rustid::common::cache::Level1Cache;
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
    fn test_intel_cache_counts() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_cache_counts(
                &cpu,
                (2, "2x "),
                (2, "2x "),
                Some((2, "2x ")),
                Some((1, "")),
            );
        });
    }

    #[test]
    fn test_intel_features() {
        with_mock_cpu(|| {
            assert!(has_ht());
            assert!(has_fpu());
            assert!(has_tsc());
            assert!(has_mmx());
            assert!(has_sse());
            assert!(has_sse2());
            assert!(has_sse3());
            assert!(has_ssse3());
            assert!(has_sse41());
            assert!(has_sse42());
            assert!(has_avx());
            assert!(has_avx2());
            assert!(has_fma());
            assert!(has_f16c());
            assert!(has_aes());
            assert!(has_cx16());
            assert!(has_rdrand());
            assert!(has_bmi1());
            assert!(has_bmi2());
        });
    }
}

mod e5_2407 {
    use super::*;

    fn with_mock_cpu(test: impl FnOnce()) {
        set_file_cpuid_provider("dump/e5-2407.txt");
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
            let brand = Cpu::detect().display_model_string();
            assert!(brand.contains("Intel"));
            assert!(brand.contains("E5-2407"));
        });
    }

    #[test]
    fn test_intel_signature() {
        with_mock_cpu(|| {
            assert_eq!(get_signature(), (0, 6, 2, 13, 7));
        });
    }

    #[test]
    fn test_intel_max_leaf() {
        with_mock_cpu(|| {
            let res = max_leaf();
            assert_eq!(res, 0xd);
        });
    }

    #[test]
    fn test_intel_topology() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.sockets.count, 1);
            assert_eq!(cpu.topology.cores.count, 4);
            assert_eq!(cpu.topology.threads.count, 4);
        });
    }

    #[test]
    fn test_intel_cache_detection() {
        with_mock_cpu(|| {
            use rustid::common::cache::CacheType;
            let cpu = Cpu::detect();
            let cache = cpu.topology.cache.expect("Expected cache to be detected");

            assert_eq!(
                cache.l1.size(),
                65536,
                "L1 should be 64KB total (32KB D + 32KB I)"
            );
            assert!(cache.l1.is_split(), "L1 cache should be split");

            if let Some(l2) = cache.l2 {
                assert_eq!(l2.kind(), CacheType::Unified);
                assert_eq!(l2.size(), 262144, "L2 should be 256KB");
                assert_eq!(l2.assoc(), 8, "L2 should be 8-way");
            }

            if let Some(l3) = cache.l3 {
                assert_eq!(l3.kind(), CacheType::Unified);
                assert_eq!(l3.size(), 10485760, "L3 should be 10MB");
                assert_eq!(l3.assoc(), 20, "L3 should be 20-way");
            }
        });
    }

    #[test]
    fn test_intel_cache_assoc() {
        with_mock_cpu(|| {
            use rustid::common::cache::Level1Cache;
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
                _ => panic!("Expected split cache"),
            }
        });
    }

    #[test]
    fn test_intel_cache_counts() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_cache_counts(
                &cpu,
                (4, "4x "),
                (4, "4x "),
                Some((4, "4x ")),
                Some((1, "")),
            );
        });
    }
}

mod amd_7950x3d {
    use super::*;

    fn with_mock_cpu(test: impl FnOnce()) {
        set_file_cpuid_provider("dump/7950x3d.txt");
        test();
    }

    #[test]
    fn test_vendor_detection() {
        with_mock_cpu(|| {
            let vendor = vendor_str();
            assert_eq!(&*vendor, VENDOR_AMD);
            assert!(is_amd());
        });
    }

    #[test]
    fn test_signature() {
        with_mock_cpu(|| {
            assert_eq!(get_signature(), (10, 15, 6, 1, 2));
        });
    }

    #[test]
    fn test_brand_string() {
        with_mock_cpu(|| {
            let brand = Cpu::detect().display_model_string();
            assert_eq!(brand, "AMD Ryzen 9 7950X3D 16-Core Processor");
        });
    }

    #[test]
    fn test_topology() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();

            assert_eq!(cpu.topology.dies.count, 2);
            assert_eq!(cpu.topology.threads.count, 32);
            assert_eq!(cpu.topology.cores.count, 16);
            assert_eq!(cpu.topology.sockets.count, 1);
        });
    }

    #[test]
    fn test_cache_counts() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_cache_counts(
                &cpu,
                (16, "16x "),
                (16, "16x "),
                Some((16, "16x ")),
                Some((2, "2x ")),
            );
        });
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
            let brand = Cpu::detect().display_model_string();
            assert!(brand.contains("AMD"));
            assert!(brand.contains("5900"));
        });
    }

    #[test]
    fn test_amd_signature() {
        with_mock_cpu(|| {
            assert_eq!(get_signature(), (10, 15, 2, 1, 2));
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
    fn test_amd_topology() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.sockets.count, 1);
            assert_eq!(cpu.topology.dies.count, 1);
            assert_eq!(cpu.topology.cores.count, 16);
            assert_eq!(cpu.topology.threads.count, 32);
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
            use rustid::common::cache::CacheType;
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
            use rustid::common::cache::Level1Cache;
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
    fn test_amd_cache_counts() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_cache_counts(
                &cpu,
                (16, "16x "),
                (16, "16x "),
                Some((16, "16x ")),
                Some((2, "2x ")),
            );
        });
    }

    #[test]
    fn test_amd_features() {
        with_mock_cpu(|| {
            assert!(has_ht());
            assert!(has_mmx());
            assert!(has_sse());
            assert!(has_sse2());
            assert!(has_sse3());
            assert!(has_ssse3());
            assert!(has_sse41());
            assert!(has_sse42());
            assert!(has_sse4a());
            assert!(has_avx());
            assert!(has_avx2());
            assert!(has_fma());
            assert!(has_f16c());
            assert!(has_aes());
            assert!(has_popcnt());
            assert!(has_amd64());
            assert!(has_x2apic());
            assert!(!has_3dnow());
            assert!(!has_3dnow_plus());
        });
    }
}

mod amd_2700u {
    use super::*;

    fn with_mock_cpu(test: impl FnOnce()) {
        set_file_cpuid_provider("dump/2700U.txt");
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
            let brand = Cpu::detect().display_model_string();
            assert!(brand.contains("2700U"));
        });
    }

    #[test]
    fn test_amd_signature() {
        with_mock_cpu(|| {
            assert_eq!(get_signature(), (8, 15, 1, 1, 0));
        });
    }

    #[test]
    fn test_amd_topology() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.threads.count, 8);
            assert_eq!(cpu.topology.cores.count, 4);
            assert_eq!(cpu.topology.sockets.count, 1);
            assert_eq!(cpu.topology.dies.count, 1);
        });
    }

    #[test]
    fn test_amd_cache_counts() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_cache_counts(
                &cpu,
                (4, "4x "),
                (4, "4x "),
                Some((4, "4x ")),
                Some((1, "")),
            );
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
            let brand = Cpu::detect().display_model_string();
            assert!(brand.contains("KX-5640") || brand.contains("ZHAOXIN"));
        });
    }

    #[test]
    fn test_zhaoxin_signature() {
        with_mock_cpu(|| {
            assert_eq!(get_signature(), (0, 7, 1, 11, 0));
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
    fn test_zhaoxin_topology() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.sockets.count, 1);
            assert_eq!(cpu.topology.cores.count, 4);
            assert_eq!(cpu.topology.threads.count, 4);
        });
    }

    #[test]
    fn test_zhaoxin_cache_detection() {
        with_mock_cpu(|| {
            use rustid::common::{Level1Cache, cache::CacheType};
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
    fn test_zhaoxin_cache_counts() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_cache_counts(&cpu, (4, "4x "), (4, "4x "), Some((1, "")), None);
        });
    }

    #[test]
    fn test_zhaoxin_features() {
        with_mock_cpu(|| {
            assert!(has_ht());
            assert!(has_sse());
            assert!(has_sse2());
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

    #[test]
    fn test_zhaoxin_padlock_features() {
        with_mock_cpu(|| {
            use rustid::x86::vendor::centaur;
            assert!(centaur::has_rng(), "KX-5640 has RNG");
            assert!(centaur::rng_enabled(), "KX-5640 rng enabled");
            assert!(centaur::has_rng2(), "KX-5640 has RNG2");
            assert!(centaur::rng2_enabled(), "KX-5640 rng2 enabled");
            assert!(centaur::has_ace(), "KX-5640 has ACE");
            assert!(centaur::ace_enabled(), "KX-5640 has ACE enabled");
            assert!(centaur::has_ace2(), "KX-5640 has ACE2");
            assert!(!centaur::ace2_enabled(), "KX-5640 has AC2 disabled");
            assert!(centaur::has_phe(), "KX-5640 has PHE");
            assert!(centaur::phe_enabled(), "KX-5640 has PHE enabled");
            assert!(centaur::has_phe2(), "KX-5640 has PHE2");
            assert!(centaur::phe2_enabled(), "KX-5640 has PHE2 enabled");
            assert!(centaur::has_pmm(), "KX-5640 has PMM");
            assert!(centaur::pmm_enabled(), "KX-5640 has PMM enabled");
            assert!(centaur::has_rsa(), "KX-5640 has RSA");
            assert!(centaur::rsa_enabled(), "KX-5640 has RSA enabled");
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
            let brand = Cpu::detect().display_model_string();
            assert!(brand.contains("C7") || !brand.is_empty());
        });
    }

    #[test]
    fn test_via_signature() {
        with_mock_cpu(|| {
            assert_eq!(get_signature(), (0, 6, 0, 10, 9));
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
    fn test_via_features() {
        with_mock_cpu(|| {
            assert!(!has_ht());
            assert!(has_sse());
            assert!(has_sse2());
            assert!(has_sse3());
        });
    }

    #[test]
    fn test_via_cache_detection() {
        with_mock_cpu(|| {
            use rustid::common::cache::CacheType;
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
    fn test_via_centaur_extended() {
        with_mock_cpu(|| {
            let res = x86_cpuid_count(0xC000_0000, 0);
            assert_eq!(res.eax, 0xC000_0002);
        });
    }

    #[test]
    fn test_via_topology() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.sockets.count, 1);
            assert_eq!(cpu.topology.cores.count, 1);
            assert_eq!(cpu.topology.threads.count, 1);
        });
    }

    #[test]
    fn test_via_padlock_features() {
        with_mock_cpu(|| {
            use rustid::x86::vendor::centaur;
            assert!(centaur::has_rng(), "C7-D has RNG");
            assert!(centaur::rng_enabled(), "C7-D has RNG enabled");
            assert!(!centaur::has_rng2(), "C7-D has no RNG2");
            assert!(centaur::has_ace(), "C7-D has ACE");
            assert!(centaur::ace_enabled(), "C7-D has ACE enabled");
            assert!(centaur::has_ace2(), "C7-D ACE2");
            assert!(centaur::ace2_enabled(), "C7-D ACE2 enabled");
            assert!(centaur::has_phe(), "C7-D has PHE");
            assert!(!centaur::has_phe2(), "C7-D does not have PHE2");
            assert!(centaur::has_pmm(), "C7-D has PMM");
        });
    }
}

mod olpc {
    use super::*;

    fn with_mock_cpu(test: impl FnOnce()) {
        set_file_cpuid_provider("dump/olpc.txt");
        test();
    }

    #[test]
    fn test_olpc_vendor_detection() {
        with_mock_cpu(|| {
            let vendor = vendor_str();
            assert_eq!(&*vendor, VENDOR_CENTAUR);
        });
    }

    #[test]
    fn test_olpc_signature() {
        with_mock_cpu(|| {
            assert_eq!(get_signature(), (0, 6, 0, 13, 0));
        });
    }

    #[test]
    fn test_olpc_topology() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.sockets.count, 1);
            assert_eq!(cpu.topology.cores.count, 1);
            assert_eq!(cpu.topology.threads.count, 1);
        });
    }

    #[test]
    fn test_olpc_padlock_features() {
        with_mock_cpu(|| {
            use rustid::x86::vendor::centaur;
            assert!(centaur::has_rng(), "C7-M has RNG");
            assert!(centaur::rng_enabled(), "C7-M has RNG enabled");
            assert!(!centaur::has_rng2(), "C7-M has no RNG2");
            assert!(centaur::has_ace(), "C7-M has ACE");
            assert!(centaur::ace_enabled(), "C7-M has ACE enabled");
            assert!(centaur::has_ace2(), "C7-M ACE2");
            assert!(centaur::ace2_enabled(), "C7-M ACE2 enabled");
            assert!(centaur::has_phe(), "C7-M has PHE");
            assert!(!centaur::has_phe2(), "C7-M does not have PHE2");
            assert!(centaur::has_pmm(), "C7-M has PMM");
        });
    }
}

mod idt_w2b {
    use super::*;

    fn with_mock_cpu(test: impl FnOnce()) {
        set_file_cpuid_provider("dump/W2B-DUMP.TXT");
        test();
    }

    #[test]
    fn test_w2b_vendor_detection() {
        with_mock_cpu(|| {
            let vendor = vendor_str();
            assert_eq!(&*vendor, VENDOR_CENTAUR);
        });
    }

    #[test]
    fn test_w2b_signature() {
        with_mock_cpu(|| {
            assert_eq!(get_signature(), (0, 5, 0, 8, 10));
        });
    }

    #[test]
    fn test_w2b_topology() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.sockets.count, 1);
            assert_eq!(cpu.topology.cores.count, 1);
            assert_eq!(cpu.topology.threads.count, 1);
        });
    }

    #[test]
    fn test_w2b_padlock_features() {
        with_mock_cpu(|| {
            use rustid::x86::vendor::centaur;
            assert!(!centaur::has_rng(), "Winchip 2B has no RNG");
            assert!(!centaur::has_rng2(), "Winchip 2B has no RNG2");
            assert!(!centaur::has_ace(), "Winchip 2B has no ACE enabled");
            assert!(!centaur::has_ace2(), "Winchip 2B ACE2 not present");
            assert!(!centaur::has_phe(), "Winchip 2B has no PHE");
            assert!(!centaur::has_phe2(), "Winchip 2B has no PHE2");
            assert!(!centaur::has_pmm(), "Winchip 2B has no PMM");
        });
    }
}

mod vortex86dx3 {
    use rustid::x86::{has_ht, has_mmx, max_extended_leaf};

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
            let brand = Cpu::detect().display_model_string();
            assert!(brand.contains("Vortex86"));
        });
    }

    #[test]
    fn test_vortex86_signature() {
        with_mock_cpu(|| {
            assert_eq!(get_signature(), (0, 6, 0, 1, 1));
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
    fn test_vortex86_topology() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.sockets.count, 1);
            assert_eq!(cpu.topology.cores.count, 1);
            assert_eq!(cpu.topology.threads.count, 1);
        });
    }

    #[test]
    fn test_vortex86_cache_detection() {
        with_mock_cpu(|| {
            use rustid::common::{Level1Cache, cache::CacheType};
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
    fn test_vortex86_features() {
        with_mock_cpu(|| {
            assert!(!has_ht());
            assert!(has_mmx());
            let fc = FeatureClass::detect();
            assert!(matches!(fc, FeatureClass::i686_SSE));
        });
    }
}

mod via_edenx2 {
    use super::*;

    fn with_mock_cpu(test: impl FnOnce()) {
        set_file_cpuid_provider("dump/edenx2.txt");
        test();
    }

    #[test]
    fn test_edenx2_vendor_detection() {
        with_mock_cpu(|| {
            let vendor = vendor_str();
            assert_eq!(&*vendor, VENDOR_CENTAUR);
        });
    }

    #[test]
    fn test_edenx2_brand_string() {
        with_mock_cpu(|| {
            let brand = Cpu::detect().display_model_string();
            assert_eq!(brand, "VIA Nano X2 U4200");
        });
    }

    #[test]
    fn test_edenx2_signature() {
        with_mock_cpu(|| {
            assert_eq!(get_signature(), (0, 6, 0, 15, 13));
        });
    }

    #[test]
    fn test_edenx2_topology() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.sockets.count, 1);
            assert_eq!(cpu.topology.cores.count, 2);
            assert_eq!(cpu.topology.threads.count, 2);
        });
    }

    #[test]
    fn test_edenx2_features() {
        with_mock_cpu(|| {
            assert!(has_mmx());
            assert!(has_sse());
            assert!(has_sse2());
            assert!(has_sse3());
            assert!(has_ssse3());
            assert!(has_sse41());
            assert!(has_amd64());
            assert!(has_cx16());
            assert!(has_popcnt());
        });
    }

    #[test]
    fn test_edenx2_padlock_features() {
        with_mock_cpu(|| {
            use rustid::x86::vendor::centaur;
            assert!(centaur::has_rng(), "Eden X2 has RNG");
            assert!(centaur::has_ace(), "Eden X2 has ACE");
            assert!(centaur::has_phe(), "Eden X2 has PHE");
            assert!(centaur::has_pmm(), "Eden X2 has PMM");
        });
    }

    #[test]
    fn test_edenx2_cache_detection() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            let cache = cpu.topology.cache.expect("Expected cache to be detected");
            assert_eq!(
                cache.l1.size(),
                131072,
                "L1 should be 128KB (64KB D + 64KB I)"
            );
        });
    }

    #[test]
    fn test_edenx2_cache_counts() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_cache_counts(&cpu, (2, "2x "), (2, "2x "), None, None);
        });
    }
}

mod intel_12700h {
    use super::*;

    fn with_mock_cpu(test: impl FnOnce()) {
        set_file_cpuid_provider("dump/12700H.txt");
        test();
    }

    #[test]
    fn test_12700h_vendor_detection() {
        with_mock_cpu(|| {
            let vendor = vendor_str();
            assert_eq!(&*vendor, VENDOR_INTEL);
            assert!(is_intel());
        });
    }

    #[test]
    fn test_12700h_signature() {
        with_mock_cpu(|| {
            assert_eq!(get_signature(), (0, 6, 9, 10, 3));
        });
    }

    #[test]
    fn test_12700h_brand_string() {
        with_mock_cpu(|| {
            let brand = Cpu::detect().display_model_string();
            assert!(brand.contains("12700H"));
        });
    }

    #[test]
    fn test_12700h_topology() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.sockets.count, 1);
            assert_eq!(cpu.topology.cores.count, 10);
            assert_eq!(cpu.topology.threads.count, 20);
        });
    }

    #[test]
    fn test_12700h_cache_detection() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            let cache = cpu.topology.cache.expect("Expected cache to be detected");
            assert!(cache.l3.is_some());
            assert_eq!(
                cache.l3.expect("L3 cache").size(),
                25165824,
                "L3 should be 24MB"
            );
        });
    }

    #[test]
    fn test_12700h_features() {
        with_mock_cpu(|| {
            assert!(has_mmx());
            assert!(has_sse());
            assert!(has_sse2());
            assert!(has_sse3());
            assert!(has_ssse3());
            assert!(has_sse41());
            assert!(has_sse42());
            assert!(has_avx());
            assert!(has_avx2());
            assert!(has_fma());
            assert!(has_aes());
        });
    }
}

mod intel_eeepc {
    use super::*;

    fn with_mock_cpu(test: impl FnOnce()) {
        set_file_cpuid_provider("dump/eeepc.txt");
        test();
    }

    #[test]
    fn test_eeepc_vendor_detection() {
        with_mock_cpu(|| {
            let vendor = vendor_str();
            assert_eq!(&*vendor, VENDOR_INTEL);
            assert!(is_intel());
        });
    }

    #[test]
    fn test_eeepc_signature() {
        with_mock_cpu(|| {
            assert_eq!(get_signature(), (0, 6, 0, 13, 8));
        });
    }

    #[test]
    fn test_eeepc_brand_string() {
        with_mock_cpu(|| {
            let brand = Cpu::detect().display_model_string();
            assert!(brand.contains("Celeron"));
        });
    }

    #[test]
    fn test_eeepc_topology() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            assert_eq!(cpu.topology.sockets.count, 1);
            assert_eq!(cpu.topology.cores.count, 1);
            assert_eq!(cpu.topology.threads.count, 1);
        });
    }

    #[test]
    fn test_eeepc_cache_detection() {
        with_mock_cpu(|| {
            let cpu = Cpu::detect();
            let cache = cpu.topology.cache.expect("Expected cache to be detected");
            assert_eq!(
                cache.l2.expect("L2").size(),
                262144,
                "Eee PC L2 should be 256KB"
            );
        });
    }

    #[test]
    fn test_eeepc_features() {
        with_mock_cpu(|| {
            assert!(has_mmx());
            assert!(has_sse());
            assert!(has_sse2());
            assert!(!has_sse3());
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
