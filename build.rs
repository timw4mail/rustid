use std::env::var;
use std::fs;
use std::process::Command;

/// Vendored cfg_aliases v0.2.1
/// See: <https://github.com/katharostech/cfg_aliases/blob/v0.2.1/src/lib.rs>
#[macro_export]
macro_rules! cfg_aliases {
    // Helper that just checks whether the CFG environment variable is set
    (@cfg_is_set $cfgname:ident) => {
        {
            let cfg_var = stringify!($cfgname).to_uppercase().replace("-", "_");
            let result = std::env::var(format!("CARGO_CFG_{}", &cfg_var)).is_ok();

            // CARGO_CFG_DEBUG_ASSERTIONS _should_ be set for when debug assertions are enabled,
            // but as of writing is not: see https://github.com/rust-lang/cargo/issues/5777
            if !result && cfg_var == "DEBUG_ASSERTIONS" {
                std::env::var("PROFILE") == Ok("debug".to_owned())
            } else {
                result
            }
        }
    };
    // Helper to check for the presense of a feature
    (@cfg_has_feature $feature:expr) => {
        {
            std::env::var(
                format!(
                    "CARGO_FEATURE_{}",
                    &stringify!($feature).to_uppercase().replace("-", "_").replace('"', "")
                )
            ).map(|x| x == "1").unwrap_or(false)
        }
    };

    // Helper that checks whether a CFG environment contains the given value
    (@cfg_contains $cfgname:ident = $cfgvalue:expr) => {
        std::env::var(
            format!(
                "CARGO_CFG_{}",
                &stringify!($cfgname).to_uppercase().replace("-", "_")
            )
        ).unwrap_or("".to_owned()).split(",").find(|x| x == &$cfgvalue).is_some()
    };

    // Emitting `any(clause1,clause2,...)`: convert to `$crate::cfg_aliases!(clause1) && $crate::cfg_aliases!(clause2) && ...`
    (
        @parser_emit
        all
        $({$($grouped:tt)+})+
    ) => {
        ($(
            ($crate::cfg_aliases!(@parser $($grouped)+))
        )&&+)
    };

    // Likewise for `all(clause1,clause2,...)`.
    (
        @parser_emit
        any
        $({$($grouped:tt)+})+
    ) => {
        ($(
            ($crate::cfg_aliases!(@parser $($grouped)+))
        )||+)
    };

    // "@clause" rules are used to parse the comma-separated lists. They munch
    // their inputs token-by-token and finally invoke an "@emit" rule when the
    // list is all grouped. The general pattern for recording the parser state
    // is:
    //
    // ```
    // $crate::cfg_aliases!(
    //    @clause $operation
    //    [{grouped-clause-1} {grouped-clause-2...}]
    //    [not-yet-parsed-tokens...]
    //    current-clause-tokens...
    // )
    // ```

    // This rule must come first in this section. It fires when the next token
    // to parse is a comma. When this happens, we take the tokens in the
    // current clause and add them to the list of grouped clauses, adding
    // delimeters so that the grouping can be easily extracted again in the
    // emission stage.
    (
        @parser_clause
        $op:ident
        [$({$($grouped:tt)+})*]
        [, $($rest:tt)*]
        $($current:tt)+
    ) => {
        $crate::cfg_aliases!(@parser_clause $op [
            $(
                {$($grouped)+}
            )*
            {$($current)+}
        ] [
            $($rest)*
        ])
    };

    // This rule comes next. It fires when the next un-parsed token is *not* a
    // comma. In this case, we add that token to the list of tokens in the
    // current clause, then move on to the next one.
    (
        @parser_clause
        $op:ident
        [$({$($grouped:tt)+})*]
        [$tok:tt $($rest:tt)*]
        $($current:tt)*
    ) => {
        $crate::cfg_aliases!(@parser_clause $op [
            $(
                {$($grouped)+}
            )*
        ] [
            $($rest)*
        ] $($current)* $tok)
    };

    // This rule fires when there are no more tokens to parse in this list. We
    // finish off the "current" token group, then delegate to the emission
    // rule.
    (
        @parser_clause
        $op:ident
        [$({$($grouped:tt)+})*]
        []
        $($current:tt)+
    ) => {
        $crate::cfg_aliases!(@parser_emit $op
            $(
                {$($grouped)+}
            )*
            {$($current)+}
        )
    };


    // `all(clause1, clause2...)` : we must parse this comma-separated list and
    // partner with `@emit all` to output a bunch of && terms.
    (
        @parser
        all($($tokens:tt)+)
    ) => {
        $crate::cfg_aliases!(@parser_clause all [] [$($tokens)+])
    };

    // Likewise for `any(clause1, clause2...)`
    (
        @parser
        any($($tokens:tt)+)
    ) => {
        $crate::cfg_aliases!(@parser_clause any [] [$($tokens)+])
    };

    // `not(clause)`: compute the inner clause, then just negate it.
    (
        @parser
        not($($tokens:tt)+)
    ) => {
        !($crate::cfg_aliases!(@parser $($tokens)+))
    };

    // `feature = value`: test for a feature.
    (@parser feature = $value:expr) => {
        $crate::cfg_aliases!(@cfg_has_feature $value)
    };
    // `param = value`: test for equality.
    (@parser $key:ident = $value:expr) => {
        $crate::cfg_aliases!(@cfg_contains $key = $value)
    };
    // Parse a lone identifier that might be an alias
    (@parser $e:ident) => {
        __cfg_aliases_matcher__!($e)
    };

    // Entrypoint that defines the matcher
    (
        @with_dollar[$dol:tt]
        $( $alias:ident : { $($config:tt)* } ),* $(,)?
    ) => {
        // Create a macro that expands other aliases and outputs any non
        // alias by checking whether that CFG value is set
        macro_rules! __cfg_aliases_matcher__ {
            // Parse config expression for the alias
            $(
                ( $alias ) => {
                    $crate::cfg_aliases!(@parser $($config)*)
                };
            )*
            // Anything that doesn't match evaluate the item
            ( $dol e:ident ) => {
                $crate::cfg_aliases!(@cfg_is_set $dol e)
            };
        }

        $(
            println!("cargo:rustc-check-cfg=cfg({})", stringify!($alias));
            if $crate::cfg_aliases!(@parser $($config)*) {
                println!("cargo:rustc-cfg={}", stringify!($alias));
            }
        )*
    };

    // Catch all that starts the macro
    ($($tokens:tt)*) => {
        $crate::cfg_aliases!(@with_dollar[$] $($tokens)*)
    }
}

fn build_haiku_gui(out_dir: String) {
    let cxx = var("CXX").unwrap_or_else(|_| "c++".to_string());
    let ar = var("AR").unwrap_or_else(|_| "ar".to_string());
    let obj_file = format!("{}/haiku_bridge.o", out_dir);
    let lib_file = format!("{}/librustid_haiku_bridge.a", out_dir);

    let compile_status = Command::new(&cxx)
        .args([
            "-c",
            "-O2",
            "-std=c++17",
            "src/gui/haiku/bridge/haiku_bridge.cpp",
            "-o",
            &obj_file,
        ])
        .status();

    if let Ok(status) = compile_status
        && status.success()
    {
        let _ = Command::new(&ar)
            .args(["crus", &lib_file, &obj_file])
            .status();
        println!("cargo:rustc-link-search=native={}", out_dir);
        println!("cargo:rustc-link-lib=static=rustid_haiku_bridge");
        println!("cargo:rustc-link-lib=be");
        println!("cargo:rustc-link-lib=tracker");
        println!("cargo:rustc-link-lib=stdc++");
        println!("cargo:rustc-link-lib=root");
    }

    println!("cargo:rerun-if-changed=src/gui/haiku/bridge/haiku_bridge.cpp");
    println!("cargo:rerun-if-changed=src/gui/haiku/bridge/haiku_bridge.h");
}

fn build_windows_gui(out_dir: String) {
    let manifest_dir = var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let arch = var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    let ico_rel_path = match arch.as_str() {
        "x86" => "assets/windows/rustid_x86.ico",
        "x86_64" => "assets/windows/rustid_x64.ico",
        "aarch64" => "assets/windows/rustid_arm64.ico",
        _ => "assets/rustid.ico",
    };
    let ico_full_path = format!("{}/{}", manifest_dir, ico_rel_path);

    let rc_file = format!("{}/rustid.rc", out_dir);
    let rc_content = format!("1 ICON \"{}\"\n", ico_full_path.replace('\\', "/"));
    let _ = fs::write(&rc_file, rc_content);

    let out_res = format!("{}/rustid_res.o", out_dir);
    let target = var("TARGET").unwrap_or_default();
    let target_windres = format!("{}-windres", target);

    let mut windres_candidates = Vec::new();
    if let Ok(env_windres) = var("WINDRES")
        && !env_windres.is_empty()
    {
        windres_candidates.push(env_windres);
    }
    windres_candidates.push(target_windres);
    match arch.as_str() {
        "aarch64" => {
            windres_candidates.push("aarch64-w64-mingw32-windres".to_string());
        }
        "x86_64" => {
            windres_candidates.push("x86_64-w64-mingw32-windres".to_string());
        }
        "x86" => {
            windres_candidates.push("i686-w64-mingw32-windres".to_string());
        }
        "arm" => {
            windres_candidates.push("armv7-w64-mingw32-windres".to_string());
        }
        _ => {}
    }
    windres_candidates.push("llvm-windres".to_string());
    windres_candidates.push("windres".to_string());

    let _ = fs::remove_file(&out_res);

    let mut compiled = false;
    for candidate in &windres_candidates {
        if candidate.is_empty() {
            continue;
        }
        let mut cmd = Command::new(candidate);
        cmd.args([
            "-I",
            &manifest_dir,
            "-i",
            &rc_file,
            "-o",
            &out_res,
            "-O",
            "coff",
        ]);
        if arch == "x86" {
            cmd.args(["-F", "pe-i386"]);
        } else if arch == "x86_64" {
            cmd.args(["-F", "pe-x86-64"]);
        } else if arch == "aarch64" {
            cmd.args(["--target", "aarch64-w64-mingw32"]);
        } else if arch == "arm" {
            cmd.args(["--target", "armv7-w64-mingw32"]);
        }
        if let Ok(status) = cmd.status()
            && status.success()
        {
            compiled = true;
            break;
        }
    }

    if compiled {
        println!("cargo:rustc-link-arg={}", out_res);
    }

    println!("cargo:rerun-if-env-changed=WINDRES");
    println!("cargo:rerun-if-changed=build-config/rustid.rc");
    println!("cargo:rerun-if-changed=assets/rustid.ico");
    println!("cargo:rerun-if-changed=assets/windows/rustid_x86.ico");
    println!("cargo:rerun-if-changed=assets/windows/rustid_x64.ico");
    println!("cargo:rerun-if-changed=assets/windows/rustid_arm64.ico");
}

fn main() {
    // Setup cfg aliases
    cfg_aliases! {
        // Runtime / Environment Model
        uefi: { target_os = "uefi" },
        dos_real: { all(target_os = "none", target_arch = "x86", not(feature = "dos32a-build")) },
        dos_ext: { all(target_os = "none", target_arch = "x86", feature = "dos32a-build") },
        dos_os: { all(target_os = "none", target_arch = "x86") },
        nostd_os: { any(target_os = "none", target_os = "uefi") },
        std_os: { not(any(target_os = "none", target_os = "uefi")) },

        // Operating System Families
        bsd: { any(target_os = "freebsd", target_os = "openbsd", target_os = "netbsd") },
        haiku_os: { target_os = "haiku" },
        linux_os: { any(target_os = "android", target_os = "linux") },
        unix_os: { any(target_os = "linux", target_os = "android", target_os = "macos", target_os = "ios", target_os = "freebsd", target_os = "openbsd", target_os = "netbsd") },
        windows_os: { target_os = "windows" },

        // CPU Architectures
        x86_cpu: { any(target_arch = "x86", target_arch = "x86_64") },
        arm_cpu: { any(target_arch = "arm", target_arch = "aarch64", target_arch = "arm64ec") },
        ppc_cpu: { any(target_arch = "powerpc", target_arch = "powerpc64") },
        riscv_cpu: { any(target_arch = "riscv32", target_arch = "riscv64") }
    }

    if var("CARGO_FEATURE_GUI").is_ok()
        && let Ok(out_dir) = var("OUT_DIR")
    {
        match var("CARGO_CFG_TARGET_OS").unwrap_or_default().as_str() {
            "haiku" => build_haiku_gui(out_dir),
            "windows" => build_windows_gui(out_dir),
            _ => (),
        }
    }
}
