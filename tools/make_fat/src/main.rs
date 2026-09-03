//! Merge multiple thin Mach-O binaries into a single universal ("fat") binary.
//!
//! Usage: make-fat <slice-1> <slice-2> [<slice-n>...] <output>
//!
//! Each input must be a thin Mach-O object/executable for a distinct
//! architecture. The last argument is the path of the resulting fat binary.

use std::env;
use std::process::ExitCode;

use fat_macho::FatWriter;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 3 {
        eprintln!("usage: make-fat <slice-1> <slice-2> [<slice-n>...] <output>");
        return ExitCode::from(2);
    }

    let (inputs, output) = args.split_at(args.len() - 1);
    let output = &output[0];

    let mut writer = FatWriter::new();
    for path in inputs {
        if let Err(err) = writer.add_file(path) {
            eprintln!("make-fat: cannot add '{}': {err}", path);
            return ExitCode::FAILURE;
        }
    }

    if writer.is_empty() {
        eprintln!("make-fat: no architectures added");
        return ExitCode::FAILURE;
    }

    if let Err(err) = writer.write_to_file(output) {
        eprintln!("make-fat: cannot write '{}': {err}", output);
        return ExitCode::FAILURE;
    }

    println!(
        "make-fat: wrote {} architecture(s) to {}",
        writer.len(),
        output
    );
    ExitCode::SUCCESS
}
