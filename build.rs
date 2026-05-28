use cfg_aliases::cfg_aliases;

fn main() {
    // Setup cfg aliases
    cfg_aliases! {
        // Platforms
        dos: { all(target_os = "none", target_arch= "x86") }
    }
}
