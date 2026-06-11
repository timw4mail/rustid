use super::sysctl::get_sysctl_int_value;
use crate::common::{DataSource, OS, TOSData};

impl TOSData for OS {
    fn get_socket_count() -> (u32, DataSource) {
        #[cfg(not(any(target_os = "freebsd", target_os = "netbsd")))]
        let key = "";

        #[cfg(target_os = "freebsd")]
        let key = "kern.smp.active";
        use crate::common::DataSource;

        #[cfg(target_os = "netbsd")]
        let key = "hw.acpi.cpu.dynamic";

        if let Some(sockets) = get_sysctl_int_value(key) {
            return (sockets, DataSource::Sysctrl(key));
        }

        (1, DataSource::DefaultValue)
    }
}
