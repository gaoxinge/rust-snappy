extern crate pkg_config;
extern crate cmake;

use std::env;
use std::fs;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let want_static = env::var("SNAPPY_SYS_STATIC").unwrap_or(String::new()) == "1";
    let from_source = env::var("SNAPPY_SYS_STATIC_FROM_SOURCE").unwrap_or(String::new()) == "1";
    if !from_source && configure_snappy(want_static) {
        return;
    }
    build_snappy();
}

fn configure_snappy(want_static: bool) -> bool {
    // ~ try pkg_config first
    if pkg_config::probe_library("snappy").is_ok() {
        return true;
    }
    // ~ then try search in statically predefined directories
    let libsnappy_file = if want_static { "libsnappy.a" } else { "libsnappy.so" };
    if let Some(path) = first_path_with_file(libsnappy_file) {
        if want_static {
            println!("cargo:rustc-link-search={}", path);
            println!("cargo:rustc-link-lib=static=snappy");
            configure_stdcpp();
        } else {
            println!("cargo:rustc-link-search=native={}", path);
            println!("cargo:rustc-link-lib=dylib=snappy");
        }
        return true;
    }
    return false;
}

fn build_snappy() {
    let out_dir = PathBuf::from(&env::var("OUT_DIR").unwrap());

    let cc = cc::Build::new().cpp(true).get_compiler();
    let mut cflags = OsString::new();
    for arg in cc.args() {
        cflags.push(arg);
        cflags.push(" ");
    }
    let output = cmake::Config::new("snappy")
        .env("CC", cc.path())
        .env("CFLAGS", cflags)
        .static_crt(true)
        .build();
    Command::new("make").current_dir(&output).status().unwrap();
    println!("cargo:rustc-link-lib=static=snappy");
    // On some machines, we see lib, on other machines, we see lib64. Add both:
    println!("cargo:rustc-link-search=native={}/lib", output.display());
    println!("cargo:rustc-link-search=native={}/lib64", output.display());
    println!("cargo:root={}", out_dir.to_string_lossy());
}

fn configure_stdcpp() {
    // From: https://github.com/alexcrichton/cc-rs/blob/master/src/lib.rs
    let target = env::var("TARGET").unwrap();
    let cpp = if target.contains("darwin") {
        Some("c++")
    } else if target.contains("windows") {
        None
    } else {
        Some("stdc++")
    };
    if let Some(cpp) = cpp {
        println!("cargo:rustc-link-lib={}", cpp);
    }
}

fn first_path_with_file(file: &str) -> Option<String> {
    // we want to look in LD_LIBRARY_PATH and then some default folders
    if let Some(ld_path) = env::var_os("LD_LIBRARY_PATH") {
        for p in env::split_paths(&ld_path) {
            if is_file_in(file, &p) {
                return p.to_str().map(|s| String::from(s))
            }
        }
    }
    for p in vec!["/usr/lib","/usr/local/lib"] {
        if is_file_in(file, &Path::new(p)) {
            return Some(String::from(p))
        }
    }
    return None
}

fn is_file_in(file: &str, folder: &Path) -> bool {
    let full = folder.join(file);
    match fs::metadata(full) {
        Ok(ref found) if found.is_file() => true,
        _ => false
    }
}
