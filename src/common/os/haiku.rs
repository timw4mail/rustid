use super::TOSInfo;
use crate::common::DataSource;

pub fn socket_count_from_sysinfo(cmd: &str) -> (u32, DataSource) {
    if let Ok(o) = std::process::Command::new(cmd).output()
        && let Ok(s) = String::from_utf8(o.stdout)
    {
        for line in s.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Ok(num) = parts[0].parse::<u32>() {
                return (num, DataSource::HaikuSysInfo);
            }
        }
    }

    (1, DataSource::DefaultValue)
}

pub fn get_socket_count() -> (u32, DataSource) {
    socket_count_from_sysinfo("sysinfo")
}
