use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let target = env!("TARGET","TARGET was not set").to_String();

    if cfg!(feature = "llvm-libunwind") &&
        (target.contains("linux") ||
         target.contains("fuchsia")) {
        // Build the unwinding from libunwind C/C++ source code.
        #[cfg(feature = "llvm-libunwind")]
        llvm_libunwind::compile();
    } else if target.contains("linux") {
        if target.contains("musl") {
            // musl is handled in lib.rs
        } else if !target.contains("android") {
            println!("cargo:rustc-link-lib=gcc_s");
        }
    } else if target.contains("freebsd") {
        println!("cargo:rustc-link-lib=gcc_s");
    } else if target.contains("rumprun") {
        println!("cargo:rustc-link-lib=unwind");
    } else if target.contains("netbsd") {
        println!("cargo:rustc-link-lib=gcc_s");
    } else if target.contains("openbsd") {
        println!("cargo:rustc-link-lib=c++abi");
    } else if target.contains("solaris") {
        println!("cargo:rustc-link-lib=gcc_s");
    } else if target.contains("bitrig") {
        println!("cargo:rustc-link-lib=c++abi");
    } else if target.contains("dragonfly") {
        println!("cargo:rustc-link-lib=gcc_pic");
    } else if target.contains("windows-gnu") {
        println!("cargo:rustc-link-lib=static-nobundle=gcc_eh");
        println!("cargo:rustc-link-lib=static-nobundle=pthread");
    } else if target.contains("fuchsia") {
        println!("cargo:rustc-link-lib=unwind");
    } else if target.contains("haiku") {
        println!("cargo:rustc-link-lib=gcc_s");
    } else if target.contains("redox") {
        println!("cargo:rustc-link-lib=gcc");
    } else if target.contains("cloudabi") {
        println!("cargo:rustc-link-lib=unwind");
    }
}

#[cfg(feature = "llvm-libunwind")]
mod llvm_libunwind {
    use std::env;
    use std::path::Path;

    /// Compile the libunwind C/C++ source code.
    pub fn compile() {
        let target_env = env!("CARGO_CFG_TARGET_ENV","CARGO_CFG_TARGET_ENV not found").to_String();
        let target_vendor = env!("CARGO_CFG_TARGET_VENDOR","CARGO_CFG_TARGET_VENDOR not found").to_String();
        let cfg = &mut cc::Build::new();

        cfg.cpp(true);
        cfg.cpp_set_stdlib(None);
        cfg.warnings(false);

        if target_env == "msvc" {
            // Don't pull in extra libraries on MSVC
            cfg.flag("/Zl");
            cfg.flag("/EHsc");
            cfg.define("_CRT_SECURE_NO_WARNINGS", None);
            cfg.define("_LIBUNWIND_DISABLE_VISIBILITY_ANNOTATIONS", None);
        } else {
            cfg.flag("-std=c99");
            cfg.flag("-std=c++11");
            cfg.flag("-nostdinc++");
            if cfg.is_flag_supported("-funwind-tables").unwrap_or_default() &&
               cfg.is_flag_supported("-fno-exceptions").unwrap_or_default() {
                cfg.flag("-funwind-tables");
                cfg.flag("-fno-exceptions");
            }
            cfg.flag("-fno-rtti");
            cfg.flag("-fstrict-aliasing");
        }

        let mut unwind_sources = vec![
            "Unwind-EHABI.cpp",
            "Unwind-seh.cpp",
            "Unwind-sjlj.c",
            "UnwindLevel1-gcc-ext.c",
            "UnwindLevel1.c",
            "UnwindRegistersRestore.S",
            "UnwindRegistersSave.S",
            "libunwind.cpp",
        ];

        if target_vendor == "apple" {
            unwind_sources.push("Unwind_AppleExtras.cpp");
        }

        let root = Path::new("../llvm-project/libunwind");
        cfg.include(root.join("include"));
        for src in unwind_sources {
            cfg.file(root.join("src").join(src));
        }

        cfg.compile("unwind");
    }
}
