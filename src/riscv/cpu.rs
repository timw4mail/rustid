//! Contains the Cpu struct for RISC-V.
use super::micro_arch::*;
use crate::common::*;
use std::collections::BTreeMap;

/// RISC-V architecture-specific data.
#[derive(Debug, Default, PartialEq)]
pub struct RiscvData {
    pub cpu_arch: CpuArch,
    pub isa_string: String,
    pub raw: BTreeMap<String, String>,
    pub midr_source: DataSource,
    pub features_source: DataSource,
}

pub type Cpu = crate::common::Cpu<RiscvData, MicroArch>;

impl TDetect for Cpu {
    fn detect() -> Self {
        let info = crate::riscv::os::detect();
        let features = super::get_all_features(&info.isa_string);

        let extra = RiscvData {
            cpu_arch: info.cpu_arch,
            isa_string: info.isa_string,
            raw: info.raw,
            midr_source: info.midr_source,
            features_source: info.features_source,
        };

        let sockets = OS::get_socket_count();
        let total_cores = info.cores.iter().map(|c| c.count).sum();
        let total_threads = info.cores.iter().map(|c| c.threads).sum();
        let topology = Topology {
            sockets,
            cores: TopologyTier::new(total_cores, sockets.source),
            threads: TopologyTier::new(total_threads, sockets.source),
            ..Default::default()
        };

        Self {
            system: OS::get_system_name(),
            vendor: info.vendor,
            model: info.model,
            topology,
            cores: info.cores,
            features,
            extra,
        }
    }
}

impl TCpuDisplay for Cpu {
    fn render_debug(&self) -> alloc::string::String {
        alloc::format!("{:#?}", self)
    }

    fn display_table_with_disp(&self, disp: &mut CpuDisplay) {
        CpuDisplay::display_riscv(self, disp);
    }
}
