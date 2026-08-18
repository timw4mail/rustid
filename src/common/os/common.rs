use alloc::string::String;

/// Normalize a string for comparison: trim and collapse runs of whitespace.
pub fn normalize_for_compare(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_was_space = false;
    for c in s.trim().chars() {
        if c.is_whitespace() {
            if !last_was_space && !out.is_empty() {
                out.push(' ');
            }
            last_was_space = true;
        } else {
            out.push(c.to_ascii_lowercase());
            last_was_space = false;
        }
    }
    out
}

/// Returns true when a DMI / device-tree / SMBIOS value is a firmware placeholder or
/// other generic string that does not identify the actual hardware.
pub fn is_generic_value(raw: &str) -> bool {
    const GENERIC: &[&str] = &[
        "to be filled by o.e.m.",
        "to be filled",
        "not specified by o.e.m.",
        "system product name",
        "system name",
        "product name",
        "all series",
        "default string",
        "not specified",
        "not applicable",
        "unknown",
        "none",
        "generic",
        "oem",
        "o.e.m.",
        "i386",
        "x86",
        "x64",
        "laptop",
    ];

    let normalized = normalize_for_compare(raw);
    normalized.is_empty() || GENERIC.contains(&normalized.as_str())
}

/// Vendor strings that identify a hypervisor rather than a physical machine.
pub fn is_known_hypervisor_vendor(vendor: &str) -> bool {
    let vendor = normalize_for_compare(vendor);
    const HYPERVISORS: &[&str] = &[
        "qemu",
        "bochs",
        "kvm",
        "vmware",
        "vmware, inc.",
        "innotek gmbh",
        "microsoft corporation",
        "xen",
        "parallels",
        "parallels software international inc.",
        "openstack foundation",
        "amazon ec2",
        "google",
    ];
    HYPERVISORS.contains(&vendor.as_str())
}
