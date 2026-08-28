use crate::common::DataSource;
#[cfg(target_os = "haiku")]
use crate::common::{OS, TOSData, TopologyTier};

/// Parses the total logical CPU count from the output of Haiku's `sysinfo` command.
pub fn parse_cpu_count_from_sysinfo(s: &str) -> (u32, DataSource) {
    for line in s.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty()
            && let Ok(num) = parts[0].parse::<u32>()
        {
            return (num, DataSource::HaikuSysinfo);
        }
    }

    (1, DataSource::DefaultValue)
}

/// Invokes `sysinfo` and retrieves the total logical CPU count.
pub fn cpu_count_from_sysinfo(cmd: &str) -> (u32, DataSource) {
    if let Ok(o) = std::process::Command::new(cmd).output()
        && let Ok(s) = String::from_utf8(o.stdout)
    {
        return parse_cpu_count_from_sysinfo(&s);
    }

    (1, DataSource::DefaultValue)
}

#[cfg(target_os = "haiku")]
impl TOSData for OS {
    fn get_socket_count() -> TopologyTier {
        let (total_cpus, source) = cpu_count_from_sysinfo("sysinfo");

        #[cfg(x86_cpu)]
        {
            let threads_per_pkg = crate::x86::cpuid_threads_per_package().max(1);
            let sockets = (total_cpus / threads_per_pkg).max(1);
            TopologyTier::new(sockets, source)
        }

        #[cfg(not(x86_cpu))]
        {
            TopologyTier::new(1, source)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cpu_count_from_sysinfo() {
        let raw = include_str!("../../../tests/cpuid/haiku-sysinfo/nanox2.txt");
        let (count, source) = parse_cpu_count_from_sysinfo(raw);
        assert_eq!(count, 2);
        assert_eq!(source, DataSource::HaikuSysinfo);

        let quad_sample =
            "4 Intel(R) Core(TM) i5-2520M CPU @ 2.50GHz, revision 000206a7\n\nCPU #0...";
        let (count, source) = parse_cpu_count_from_sysinfo(quad_sample);
        assert_eq!(count, 4);
        assert_eq!(source, DataSource::HaikuSysinfo);

        let fallback_sample = "invalid output";
        let (count, source) = parse_cpu_count_from_sysinfo(fallback_sample);
        assert_eq!(count, 1);
        assert_eq!(source, DataSource::DefaultValue);
    }
}
