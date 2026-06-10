use std::collections::{BTreeMap, HashMap};
use std::process::Command;

use crate::common::DataSource;

/// Return a map of sysctl output with keys matching the `prefix`,
/// with the values parsed as u32. The keys are the original sysctl keys
/// after the `strip_prefix` value is removed
pub fn get_int_sysctl_map(prefix: &str, strip_prefix: &str) -> HashMap<String, u32> {
    let mut map: HashMap<String, u32> = HashMap::new();

    for (key, value) in get_sysctl_map_by_prefix(prefix, strip_prefix) {
        if let Ok(v) = value.parse::<u32>() {
            map.insert(key, v);
        }
    }

    map
}

/// Get the value for sysctl key `name` and attempt to parse the value
/// as a `u32`.
/// Returns Some(value) on success, None on parse failure or empty value
pub fn get_sysctl_int_value(name: &str) -> Option<u32> {
    if let Some(v) = get_sysctl_value(name)
        && let Ok(v) = v.parse::<u32>()
    {
        Some(v)
    } else {
        None
    }
}

/// Attempt to get the value for sysctl key `name`
///
/// Returns Some(value) on success, None on empty or missing value
pub fn get_sysctl_value(name: &str) -> Option<String> {
    if let Some(stdout) = cmd_output_to_string("sysctl", name) {
        for line in stdout.lines() {
            let line = line.trim();
            if let Some((key, value)) = line.split_once(':')
                && key.trim() == name
            {
                let value = value.trim();

                return Some(String::from(value));
            }
        }
    }

    None
}

pub fn get_socket_count() -> (u32, DataSource) {
    #[cfg(not(any(target_os = "freebsd", target_os = "netbsd")))]
    let key = "";

    #[cfg(target_os = "freebsd")]
    let key = "kern.smp.cpus";

    #[cfg(target_os = "netbsd")]
    let key = "hw.acpi.cpu.dynamic";

    if let Some(sockets) = get_sysctl_int_value(key) {
        return (sockets, DataSource::Sysctrl);
    }

    (1, DataSource::DefaultValue)
}

fn get_sysctl_map_by_prefix(prefix: &str, strip_prefix: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    if let Some(stdout) = cmd_output_to_string("sysctl", prefix) {
        for line in stdout.lines() {
            let line = line.trim();
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();

                let new_key = key
                    .strip_prefix(strip_prefix)
                    .expect("sysctl key doesn't start with expected prefix")
                    .to_lowercase();

                map.insert(new_key, String::from(value));
            }
        }
    }

    map
}

/// Parses the output of `sysctl -a` and converts it into a map of keys and values
pub fn get_full_raw_sysctl_map() -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();

    if let Some(stdout) = cmd_output_to_string("sysctl", "-a") {
        for line in stdout.lines() {
            let line = line.trim();
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();

                map.insert(String::from(key), String::from(value));
            }
        }
    }

    map
}

fn cmd_output_to_string(command: &str, arg: &str) -> Option<String> {
    if let Ok(output) = Command::new(command).arg(arg).output()
        && let Ok(stdout) = String::from_utf8(output.stdout)
    {
        Some(stdout)
    } else {
        None
    }
}
