use crate::common::{DataSource, OS, TOSData, TopologyTier};

pub fn socket_count_from_sysinfo(cmd: &str) -> (u32, DataSource) {
    if let Ok(o) = std::process::Command::new(cmd).output()
        && let Ok(s) = String::from_utf8(o.stdout)
    {
        for line in s.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Ok(num) = parts[0].parse::<u32>() {
                return (num, DataSource::HaikuSysinfo);
            }
        }
    }

    (1, DataSource::DefaultValue)
}

impl TOSData for OS {
    fn get_socket_count() -> TopologyTier {
        let (count, source) = socket_count_from_sysinfo("sysinfo");

        TopologyTier::new(count, source)
    }
}
