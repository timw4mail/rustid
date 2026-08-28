//! Contains the Cpu struct for ARM.
use super::micro_arch::*;
use super::*;
use crate::common::*;
use std::collections::{BTreeMap, HashSet};

/// ARM architecture-specific data.
#[derive(Debug, Default, PartialEq)]
pub struct ArmData {
    pub midrs: HashSet<Midr>,
    pub cpu_arch: CpuArch,
    pub soc_model: Option<String>,
    pub raw: BTreeMap<String, String>,
    pub midr_source: DataSource,
    pub features_source: DataSource,
}

pub type Cpu = crate::common::Cpu<ArmData, MicroArch>;

impl TDetect for Cpu {
    fn detect() -> Self {
        let info = crate::arm::os::detect();
        let features = super::get_all_features();

        let extra = ArmData {
            midrs: info.midrs,
            cpu_arch: info.cpu_arch,
            soc_model: OS::get_soc(),
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

impl TArmCpu for Cpu {
    fn model(&self) -> Option<&str> {
        if self.model.is_empty() {
            None
        } else {
            Some(&self.model)
        }
    }

    fn vendor(&self) -> &str {
        &self.vendor
    }
}
