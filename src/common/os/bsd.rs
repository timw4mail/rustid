use super::sysctl::{get_sysctl_int_value, get_sysctl_value};
use crate::common::{DataSource, OS, TOSData, TopologyTier};

impl TOSData for OS {
    fn get_soc() -> Option<String> {
        #[cfg(target_os = "freebsd")]
        {
            if let Some(raw) = get_sysctl_value("hw.fdt.compatible") {
                let parts = raw.split(",");
                if let Some(last) = parts.last() {
                    return Some(String::from(last));
                }
            }
            None
        }

        #[cfg(not(target_os = "freebsd"))]
        None
    }

    fn get_system_name() -> Option<String> {
        #[cfg(target_os = "netbsd")]
        {
            if let Some(model) = get_sysctl_value("hw.model") {
                return Some(String::from(model.trim()));
            }
            None
        }

        #[cfg(target_os = "freebsd")]
        {
            if let Some(sys) = get_sysctl_value("hw.fdt.model") {
                return Some(String::from(sys.trim()));
            }
            None
        }

        #[cfg(not(any(target_os = "netbsd", target_os = "freebsd")))]
        None
    }

    fn get_socket_count() -> TopologyTier {
        #[cfg(not(any(target_os = "freebsd", target_os = "netbsd")))]
        let key = "";

        #[cfg(target_os = "freebsd")]
        let key = "kern.smp.active";

        #[cfg(target_os = "netbsd")]
        let key = "hw.acpi.cpu.dynamic";

        if let Some(sockets) = get_sysctl_int_value(key) {
            return TopologyTier::new(sockets, DataSource::Sysctrl(key));
        }

        TopologyTier::default()
    }
}
