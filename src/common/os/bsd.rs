use super::is_generic_value;
use super::sysctl::{get_sysctl_int_value, get_sysctl_value};
use crate::common::{DataSource, OS, SystemInfo, TOSData, TopologyTier};

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

    fn get_system_name() -> Option<SystemInfo> {
        #[cfg(any(target_os = "netbsd", target_os = "openbsd"))]
        {
            if let Some(model) = get_sysctl_value("hw.model") {
                let model = model.trim();
                if !is_generic_value(model) {
                    let vendor = get_sysctl_value("hw.vendor");
                    return Some(SystemInfo::new(
                        vendor,
                        DataSource::Sysctrl("hw.vendor"),
                        Some(model.to_string()),
                        DataSource::Sysctrl("hw.model"),
                    ));
                }
            }
            None
        }

        #[cfg(target_os = "freebsd")]
        {
            if let Some(sys) = get_sysctl_value("hw.fdt.model") {
                let sys = sys.trim();
                if !is_generic_value(sys) {
                    return Some(SystemInfo::from_model(
                        sys,
                        DataSource::Sysctrl("hw.fdt.model"),
                    ));
                }
            }
            None
        }

        #[cfg(not(bsd))]
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
