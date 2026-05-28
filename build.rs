use cfg_aliases::cfg_aliases;

fn main() {
    // Setup cfg aliases
    cfg_aliases! {
        dos: { all(target_os = "none", target_arch= "x86") },
        arm_cpu: { any(target_arch = "arm", target_arch="aarch64", target_arch="arm64ec") },
        ppc_cpu: { any(target_arch = "powerpc", target_arch = "powerpc64") },
        x86_cpu: { any(target_arch = "x86", target_arch = "x86_64") }
    }
}
