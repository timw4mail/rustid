use std::collections::{BTreeMap, HashMap};
use std::process::Command;

pub fn get_int_sysctl_map(prefix: &str, strip_prefix: &str) -> HashMap<String, u32> {
    let mut map: HashMap<String, u32> = HashMap::new();

    for (key, value) in get_sysctl_map_by_prefix(prefix, strip_prefix) {
        if let Ok(v) = value.parse::<u32>() {
            map.insert(key.clone(), v);
        }
    }

    map
}

fn get_sysctl_map_by_prefix(prefix: &str, strip_prefix: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    if let Ok(output) = Command::new("sysctl").arg(prefix).output()
        && let Ok(stdout) = String::from_utf8(output.stdout)
    {
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

pub fn get_full_raw_sysctl_map() -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();

    if let Ok(output) = Command::new("sysctl").arg("-a").output()
        && let Ok(stdout) = String::from_utf8(output.stdout)
    {
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
