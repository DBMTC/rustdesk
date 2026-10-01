use std::{
    convert::TryFrom,
    env, fs,
    path::{Path, PathBuf},
    println,
};

fn pc_names(name: &str) -> Vec<&str> {
    match name {
        "libvpx" => vec!["vpx"],
        // Debian's libyuv-dev ships the library and headers with no .pc file.
        "libyuv" => vec!["libyuv", "yuv"],
        other => vec![other],
    }
}

fn dev_package(name: &str) -> &'static str {
    match name {
        "libvpx" => "libvpx-dev",
        "aom" => "libaom-dev",
        "libyuv" => "libyuv-dev",
        _ => "the matching -dev package",
    }
}

/// Path inside the target sysroot. A native build uses the host root.
fn in_sysroot(absolute: &str) -> PathBuf {
    match env::var("PKG_CONFIG_SYSROOT_DIR") {
        Ok(root) if !root.is_empty() => Path::new(&root).join(absolute.trim_start_matches('/')),
        _ => PathBuf::from(absolute),
    }
}

/// Debian installs libyuv without a pkg-config file. The shared library and
/// headers from libyuv-dev are still enough to link it.
fn link_unpackaged_libyuv() -> Option<Vec<PathBuf>> {
    let header = in_sysroot("/usr/include/libyuv.h");
    if !header.is_file() {
        return None;
    }
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let multiarch = match arch.as_str() {
        "x86_64" => Some("x86_64-linux-gnu"),
        "aarch64" => Some("aarch64-linux-gnu"),
        "arm" => Some("arm-linux-gnueabihf"),
        _ => None,
    };
    let mut dirs = Vec::new();
    if let Some(tuple) = multiarch {
        dirs.push(in_sysroot(&format!("/usr/lib/{tuple}")));
    }
    dirs.push(in_sysroot("/usr/lib"));
    dirs.push(in_sysroot("/usr/lib64"));
    for dir in &dirs {
        if dir.join("libyuv.so").is_file() {
            println!("cargo:rustc-link-search=native={}", dir.display());
            println!("cargo:rustc-link-lib=dylib=yuv");
            return Some(vec![in_sysroot("/usr/include")]);
        }
    }
    None
}

fn link_pkg_config(name: &str) -> Vec<PathBuf> {
    for pc_name in pc_names(name) {
        if let Ok(lib) = pkg_config::probe_library(pc_name) {
            return lib.include_paths;
        }
    }
    if name == "libyuv" {
        if let Some(paths) = link_unpackaged_libyuv() {
            return paths;
        }
    }
    let tried = pc_names(name).join(", ");
    panic!(
        "unable to find '{}' from the installed development package.\n            install {} so this build can link the system library. Bundled codec sources are not compiled.",
        tried,
        dev_package(name)
    );
}

/// Link vcpkg package.
fn link_vcpkg(mut path: PathBuf, name: &str) -> PathBuf {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let mut target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_arch == "x86_64" {
        target_arch = "x64".to_owned();
    } else if target_arch == "x86" {
        target_arch = "x86".to_owned();
    } else if target_arch == "loongarch64" {
        target_arch = "loongarch64".to_owned();
    } else if target_arch == "aarch64" {
        target_arch = "arm64".to_owned();
    } else {
        target_arch = "arm".to_owned();
    }
    let mut target = if target_os == "macos" {
        if target_arch == "x64" {
            "x64-osx".to_owned()
        } else if target_arch == "arm64" {
            "arm64-osx".to_owned()
        } else {
            format!("{}-{}", target_arch, target_os)
        }
    } else if target_os == "windows" {
        format!("{}-windows-static", target_arch)
    } else {
        format!("{}-{}", target_arch, target_os)
    };
    if target_arch == "x86" {
        target = target.replace("x64", "x86");
    }
    println!("cargo:info={}", target);
    if let Ok(vcpkg_root) = std::env::var("VCPKG_INSTALLED_ROOT") {
        path = vcpkg_root.into();
    } else {
        path.push("installed");
    }
    path.push(target);
    println!(
        "cargo:rustc-link-lib=static={}",
        name.trim_start_matches("lib")
    );
    println!(
        "cargo:rustc-link-search={}",
        path.join("lib").to_str().unwrap()
    );
    let include = path.join("include");
    println!("cargo:include={}", include.to_str().unwrap());
    include
}

/// Link homebrew package(for Mac M1).
fn link_homebrew_m1(name: &str) -> PathBuf {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    if target_os != "macos" || target_arch != "aarch64" {
        panic!("Couldn't find VCPKG_ROOT, also can't fallback to homebrew because it's only for macos aarch64.");
    }
    let mut path = PathBuf::from("/opt/homebrew/Cellar");
    path.push(name);
    let entries = if let Ok(dir) = std::fs::read_dir(&path) {
        dir
    } else {
        panic!("Could not find package in {}. Make sure your homebrew and package {} are all installed.", path.to_str().unwrap(),&name);
    };
    let mut directories = entries
        .into_iter()
        .filter(|x| x.is_ok())
        .map(|x| x.unwrap().path())
        .filter(|x| x.is_dir())
        .collect::<Vec<_>>();
    // Find the newest version.
    directories.sort_unstable();
    if directories.is_empty() {
        panic!(
            "There's no installed version of {} in /opt/homebrew/Cellar",
            name
        );
    }
    path.push(directories.pop().unwrap());
    // Link the library.
    println!(
        "cargo:rustc-link-lib=static={}",
        name.trim_start_matches("lib")
    );
    // Add the library path.
    println!(
        "cargo:rustc-link-search={}",
        path.join("lib").to_str().unwrap()
    );
    // Add the include path.
    let include = path.join("include");
    println!("cargo:include={}", include.to_str().unwrap());
    include
}

/// Find package. On Linux this is the installed library and its -dev headers.
/// Elsewhere it tries vcpkg, then homebrew (currently only for Mac M1).
/// Set NO_PKG_CONFIG_<name>=1 to skip the system package on Linux.
fn find_package(name: &str) -> Vec<PathBuf> {
    let no_pkg_config_var_name = format!("NO_PKG_CONFIG_{name}");
    println!("cargo:rerun-if-env-changed={no_pkg_config_var_name}");
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "linux" && std::env::var(&no_pkg_config_var_name).as_deref() != Ok("1") {
        link_pkg_config(name)
    } else if let Ok(vcpkg_root) = std::env::var("VCPKG_ROOT") {
        vec![link_vcpkg(vcpkg_root.into(), name)]
    } else {
        // Try using homebrew
        vec![link_homebrew_m1(name)]
    }
}

fn generate_bindings(
    ffi_header: &Path,
    include_paths: &[PathBuf],
    ffi_rs: &Path,
    exact_file: &Path,
    regex: &str,
) {
    let mut b = bindgen::builder()
        .header(ffi_header.to_str().unwrap())
        .allowlist_type(regex)
        .allowlist_var(regex)
        .allowlist_function(regex)
        .rustified_enum(regex)
        .trust_clang_mangling(false)
        .layout_tests(false) // breaks 32/64-bit compat
        .generate_comments(false); // comments have prefix /*!\

    for dir in include_paths {
        b = b.clang_arg(format!("-I{}", dir.display()));
    }

    b.generate().unwrap().write_to_file(ffi_rs).unwrap();
    if let Some(parent) = exact_file.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::copy(ffi_rs, exact_file).ok(); // ignore failure
}

fn gen_vcpkg_package(package: &str, ffi_header: &str, generated: &str, regex: &str) {
    let includes = find_package(package);
    let src_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let src_dir = Path::new(&src_dir);
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    let ffi_header = src_dir.join("src").join("bindings").join(ffi_header);
    println!("rerun-if-changed={}", ffi_header.display());
    for dir in &includes {
        println!("rerun-if-changed={}", dir.display());
    }

    let ffi_rs = out_dir.join(generated);
    let exact_file = src_dir.join("generated").join(generated);
    generate_bindings(&ffi_header, &includes, &ffi_rs, &exact_file, regex);
}

// If you have problems installing ffmpeg, you can download $VCPKG_ROOT/installed from ci
// Linux require link in hwcodec
/*
fn ffmpeg() {
    // ffmpeg
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let static_libs = vec!["avcodec", "avutil", "avformat"];
    static_libs.iter().for_each(|lib| {
        find_package(lib);
    });
    if target_os == "windows" {
        println!("cargo:rustc-link-lib=static=libmfx");
    }

    // os
    let dyn_libs: Vec<&str> = if target_os == "windows" {
        ["User32", "bcrypt", "ole32", "advapi32"].to_vec()
    } else if target_os == "linux" {
        let mut v = ["va", "va-drm", "va-x11", "vdpau", "X11", "stdc++"].to_vec();
        if target_arch == "x86_64" {
            v.push("z");
        }
        v
    } else if target_os == "macos" || target_os == "ios" {
        ["c++", "m"].to_vec()
    } else if target_os == "android" {
        ["z", "m", "android", "atomic"].to_vec()
    } else {
        panic!("unsupported os");
    };
    dyn_libs
        .iter()
        .map(|lib| println!("cargo:rustc-link-lib={}", lib))
        .count();

    if target_os == "macos" || target_os == "ios" {
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=CoreVideo");
        println!("cargo:rustc-link-lib=framework=CoreMedia");
        println!("cargo:rustc-link-lib=framework=VideoToolbox");
        println!("cargo:rustc-link-lib=framework=AVFoundation");
    }
}
*/

/// ABI version from the installed -dev headers, so encoder init matches the linked library.
fn export_linked_abi(header: &str, macro_name: &str, env_key: &str, includes: &[PathBuf]) {
    let value = read_header_macro(header, macro_name, includes).unwrap_or_else(|| {
        panic!(
            "{} from the installed development package does not define {}. This build does not compile a bundled codec.",
            header, macro_name
        )
    });
    println!("cargo:rustc-env={env_key}={value}");
}

fn read_header_macro(header: &str, macro_name: &str, includes: &[PathBuf]) -> Option<u32> {
    let target = env::var("TARGET").unwrap_or_default();
    let specific = format!("CC_{}", target.replace('-', "_"));
    let cc = env::var(&specific)
        .or_else(|_| env::var("CC"))
        .unwrap_or_else(|_| "cc".to_owned());
    let mut cmd = std::process::Command::new(cc);
    cmd.arg("-E").arg("-P");
    for dir in includes {
        cmd.arg(format!("-I{}", dir.display()));
    }
    cmd.arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    let mut child = cmd.spawn().ok()?;
    {
        let stdin = child.stdin.as_mut()?;
        let src = format!("#include <{header}>\n{macro_name}\n");
        std::io::Write::write_all(stdin, src.as_bytes()).ok()?;
    }
    let out = child.wait_with_output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let expr = text.lines().last()?.trim();
    eval_int_expr(expr)
}

fn eval_int_expr(expr: &str) -> Option<u32> {
    fn eval(tokens: &mut &str) -> Option<i64> {
        let mut acc = eval_term(tokens)?;
        loop {
            let s = tokens.trim_start();
            if let Some(rest) = s.strip_prefix('+') {
                *tokens = rest;
                acc += eval_term(tokens)?;
            } else {
                *tokens = s;
                break;
            }
        }
        Some(acc)
    }
    fn eval_term(tokens: &mut &str) -> Option<i64> {
        let s = tokens.trim_start();
        if let Some(rest) = s.strip_prefix('(') {
            *tokens = rest;
            let v = eval(tokens)?;
            let s = tokens.trim_start();
            *tokens = s.strip_prefix(')')?;
            return Some(v);
        }
        let s = tokens.trim_start();
        let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            return None;
        }
        *tokens = &s[digits.len()..];
        digits.parse().ok()
    }
    let mut tokens = expr;
    let value = eval(&mut tokens)?;
    if !tokens.trim().is_empty() {
        return None;
    }
    u32::try_from(value).ok()
}

fn install_one_ffi(ffi_header: &str, generated: &str, regex: &str, includes: &[PathBuf]) {
    let src_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let pregenerated = src_dir.join("generated").join(generated);
    let ffi_rs = out_dir.join(generated);
    println!("cargo:rerun-if-env-changed=SCRAP_REGEN_BINDINGS");
    if pregenerated.exists() && env::var("SCRAP_REGEN_BINDINGS").is_err() {
        let width = env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap_or_default();
        if width != "64" {
            panic!(
                "checked-in codec bindings are for 64-bit targets. This build is {}-bit. Set SCRAP_REGEN_BINDINGS=1 to generate bindings for this target (the build host needs libclang).",
                width
            );
        }
        println!("cargo:rerun-if-changed={}", pregenerated.display());
        fs::copy(&pregenerated, &ffi_rs)
            .unwrap_or_else(|e| panic!("copy {}: {}", pregenerated.display(), e));
        return;
    }
    let header = src_dir.join("src").join("bindings").join(ffi_header);
    println!("cargo:rerun-if-changed={}", header.display());
    generate_bindings(&header, includes, &ffi_rs, &pregenerated, regex);
}

fn main() {
    // in this crate, these are also valid configurations
    println!("cargo:rustc-check-cfg=cfg(dxgi,quartz,x11)");

    // there is problem with cfg(target_os) in build.rs, so use our workaround
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    // note: all link symbol names in x86 (32-bit) are prefixed wth "_".
    // run "rustup show" to show current default toolchain, if it is stable-x86-pc-windows-msvc,
    // please install x64 toolchain by "rustup toolchain install stable-x86_64-pc-windows-msvc",
    // then set x64 to default by "rustup default stable-x86_64-pc-windows-msvc"
    let target = target_build_utils::TargetInfo::new();
    if target.unwrap().target_pointer_width() != "64" {
        // panic!("Only support 64bit system");
    }
    env::remove_var("CARGO_CFG_TARGET_FEATURE");
    env::set_var("CARGO_CFG_TARGET_FEATURE", "crt-static");

    if target_os == "linux" {
        // Shared libraries and headers from the distro -dev packages.
        let vpx_inc = find_package("libvpx");
        let aom_inc = find_package("aom");
        let yuv_inc = find_package("libyuv");
        export_linked_abi(
            "vpx/vpx_encoder.h",
            "VPX_ENCODER_ABI_VERSION",
            "LINKED_VPX_ENCODER_ABI",
            &vpx_inc,
        );
        export_linked_abi(
            "vpx/vpx_decoder.h",
            "VPX_DECODER_ABI_VERSION",
            "LINKED_VPX_DECODER_ABI",
            &vpx_inc,
        );
        export_linked_abi(
            "aom/aom_encoder.h",
            "AOM_ENCODER_ABI_VERSION",
            "LINKED_AOM_ENCODER_ABI",
            &aom_inc,
        );
        export_linked_abi(
            "aom/aom_decoder.h",
            "AOM_DECODER_ABI_VERSION",
            "LINKED_AOM_DECODER_ABI",
            &aom_inc,
        );
        install_one_ffi("vpx_ffi.h", "vpx_ffi.rs", "^[vV].*", &vpx_inc);
        install_one_ffi("aom_ffi.h", "aom_ffi.rs", "^(aom|AOM|OBU|AV1).*", &aom_inc);
        install_one_ffi("yuv_ffi.h", "yuv_ffi.rs", ".*", &yuv_inc);
    } else {
        find_package("libyuv");
        gen_vcpkg_package("libvpx", "vpx_ffi.h", "vpx_ffi.rs", "^[vV].*");
        gen_vcpkg_package("aom", "aom_ffi.h", "aom_ffi.rs", "^(aom|AOM|OBU|AV1).*");
        gen_vcpkg_package("libyuv", "yuv_ffi.h", "yuv_ffi.rs", ".*");
    }
    // ffmpeg();

    if target_os == "ios" {
        // nothing
    } else if target_os == "android" {
        println!("cargo:rustc-cfg=android");
    } else if cfg!(windows) {
        // The first choice is Windows because DXGI is amazing.
        println!("cargo:rustc-cfg=dxgi");
    } else if cfg!(target_os = "macos") {
        // Quartz is second because macOS is the (annoying) exception.
        println!("cargo:rustc-cfg=quartz");
    } else if cfg!(all(unix, feature = "x11")) {
        println!("cargo:rustc-cfg=x11");
    }
}
