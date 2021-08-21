use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    compile_webview();
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    compile_gtk_helper();
    #[cfg(target_os = "macos")]
    compile_cocoa_helper();
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn compile_gtk_helper() {
    let cflags = std::process::Command::new("pkg-config")
        .args(&["--cflags", "gtk+-3.0"])
        .output()
        .expect("Needs pkg-config and gtk installed");
    let cflags = String::from_utf8_lossy(&cflags.stdout).to_string();
    let cflags: Vec<&str> = cflags.split_ascii_whitespace().collect();
    let mut build = cc::Build::new();
    build.file("src/gtk_helper.c");
    for flag in cflags {
        build.flag(flag);
    }
    build.compile("gtkwid");
}

#[cfg(target_os = "macos")]
fn compile_cocoa_helper() {
    let mut build = cc::Build::new();
    build.file("src/cocoa_helper.m");
    build.compile("cocoa");
}

fn compile_webview() {
    println!("cargo:rerun-if-changed=webview/webview.h");
    println!("cargo:rerun-if-changed=webview/webview.cc");

    let target = env::var("TARGET").unwrap();
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    Command::new("git")
        .args(&["submodule", "update", "--init", "--recursive"])
        .current_dir(&manifest_dir)
        .status()
        .expect("Git is needed to retrieve the fltk source files!");

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .file("webview/webview.cc")
        .flag_if_supported("-w");

    if target.contains("windows") {
        if target.contains("gnu")
            build.flag("-std=c++17");
        else {
            build.flag("/std:c++17");
        }
        build.include("webview/script");

        for &lib in &[
            "windowsapp",
            "user32",
            "oleaut32",
            "ole32",
            "version",
            "shell32",
        ] {
            println!("cargo:rustc-link-lib={}", lib);
        }

        let wv_arch = if target.contains("x86_64") {
            "x64"
        } else {
            "x86"
        };

        let mut wv_path = manifest_dir;
        wv_path.push("webview/script/microsoft.web.webview2.1.0.664.37/build/native");
        wv_path.push(wv_arch);
        let webview2_dir = wv_path.as_path().to_str().unwrap();

        println!("cargo:rustc-link-search={}", webview2_dir);
        println!("cargo:rustc-link-lib=WebView2LoaderStatic");
    } else if target.contains("apple") {
        build.flag("-std=c++11");

        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=WebKit");
    } else if target.contains("linux") || target.contains("bsd") {
        let lib = pkg_config::Config::new()
            .atleast_version("2.8")
            .probe("webkit2gtk-4.0")
            .unwrap();

        for path in lib.include_paths {
            build.include(path);
        }
    } else {
        panic!("Unsupported platform");
    }

    build.compile("webview");
}
