use std::collections::HashMap;
use std::process::Command;

pub fn get_int_sysconf_map(prefix: &str, strip_prefix: &str) -> HashMap<String, u32> {
    let mut map: HashMap<String, u32> = HashMap::new();

    if let Ok(output) = Command::new("sysctl").arg(prefix).output()
        && let Ok(stdout) = String::from_utf8(output.stdout)
    {
        for line in stdout.lines() {
            let line = line.trim();
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();

                if let Ok(v) = value.parse::<u32>() {
                    let new_key = key
                        .strip_prefix(strip_prefix)
                        .expect("sysctl key doesn't start with expected prefix")
                        .to_lowercase();
                    map.insert(new_key, v);
                }
            }
        }
    }

    map
}
