#[test]
#[cfg(feature = "dos-build")]
fn test_dos_binary_size() {
    use std::path::Path;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Can't find repo folder");

    let binaries = ["rustid.exe", "debug.exe", "dump.exe"];

    for binary in binaries {
        let binary_path = Path::new(&manifest_dir).join(binary);

        assert!(
            binary_path.exists(),
            "DOS binaries not found. Build with: just build-dos"
        );

        let metadata = std::fs::metadata(&binary_path).expect("Failed to read binary metadata");
        let size = metadata.len();

        const MAX_SIZE: u64 = 128 * 1024; // 128KB (Multi-segment support)
        assert!(
            size < MAX_SIZE,
            "{} is {} bytes, exceeds 64KB limit ({} bytes)",
            binary,
            size,
            MAX_SIZE
        );
    }
}
