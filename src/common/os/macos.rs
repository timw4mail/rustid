use crate::common::{DataSource, OS, TDetect, TOSData, TopologyCount, TopologyTier};

use super::sysctl::*;

impl TDetect for TopologyCount {
    fn detect() -> Self {
        let sockets = OS::get_socket_count();

        TopologyCount {
            sockets,
            cores: get_core_count(),
            threads: get_thread_count(),
            source: DataSource::Sysctrl("machdep.cpu.*, hw.packages"),
        }
    }
}

impl TOSData for OS {
    fn get_socket_count() -> TopologyTier {
        let hw_packages = get_sysctl_int_value("hw.packages");

        match hw_packages {
            Some(packages) => TopologyTier::new(packages, DataSource::Sysctrl("hw.packages")),
            None => {
                let map = get_int_sysctl_map("machdep.cpu", "machdep.cpu.");
                let cores_per_package = map.get("cores_per_package");
                let core_count = map.get("core_count");

                if let Some(cores_per) = cores_per_package
                    && let Some(core_count) = core_count
                {
                    let sockets = if cores_per >= core_count {
                        1
                    } else {
                        core_count / cores_per
                    };

                    return TopologyTier::new(sockets, DataSource::Sysctrl("machdep.cpu.*"));
                }

                TopologyTier::default()
            }
        }
    }
}

pub fn get_core_count() -> u32 {
    get_sysctl_int_value("machdep.cpu.core_count").unwrap_or(1)
}

pub fn get_thread_count() -> u32 {
    get_sysctl_int_value("machdep.cpu.thread_count").unwrap_or(1)
}
