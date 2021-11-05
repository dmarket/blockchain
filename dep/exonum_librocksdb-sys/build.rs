extern crate cc;
extern crate pkg_config;

use pkg_config::probe_library;
use std::process::Command;
use std::fs::read_dir;
use std::env::var;

fn link(name: &str, bundled: bool) {
    let target = var("TARGET").unwrap();
    let target: Vec<_> = target.split('-').collect();
    if target.get(2) == Some(&"windows") {
        println!("cargo:rustc-link-lib=dylib={}", name);
        if bundled && target.get(3) == Some(&"gnu") {
            let dir = var("CARGO_MANIFEST_DIR").unwrap();
            println!("cargo:rustc-link-search=native={}/{}", dir, target[0]);
        }
    }
}

fn build_rocksdb() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=rocksdb/");

    let mut build = cc::Build::new();
    build.include("rocksdb/include/");
    build.include("rocksdb/");
    build.include("rocksdb/third-party/gtest-1.7.0/fused-src/");
    build.include("snappy/");
    build.include(".");

    build.opt_level(3);

    build.define("NDEBUG", Some("1"));
    build.define("SNAPPY", Some("1"));

    build.warnings(false);
    build.extra_warnings(false);

    let mut lib_sources = include_str!("rocksdb_lib_sources.txt")
        .split(" ")
        .collect::<Vec<&'static str>>();

    // We have a pregenerated a version of build_version.cc in the local directory
    lib_sources = lib_sources
        .iter()
        .cloned()
        .filter(|file| *file != "util/build_version.cc")
        .collect::<Vec<&'static str>>();

    if cfg!(target_os = "macos") {
        build.define("OS_MACOSX", Some("1"));
        build.define("ROCKSDB_PLATFORM_POSIX", Some("1"));
        build.define("ROCKSDB_LIB_IO_POSIX", Some("1"));

    }
    if cfg!(target_os = "linux") {
        build.define("OS_LINUX", Some("1"));
        build.define("ZLIB",Some("1"));
        build.define("ROCKSDB_PLATFORM_POSIX", Some("1"));
        build.define("ROCKSDB_LIB_IO_POSIX", Some("1"));
        // COMMON_FLAGS="$COMMON_FLAGS -fno-builtin-memcmp"
    }
    if cfg!(target_os = "freebsd") {
        build.define("OS_FREEBSD", Some("1"));
        build.define("ROCKSDB_PLATFORM_POSIX", Some("1"));
        build.define("ROCKSDB_LIB_IO_POSIX", Some("1"));
    }

    if cfg!(windows) {
        link("rpcrt4", false);
        build.define("OS_WIN", Some("1"));
        build.define("NOMINMAX", Some("1"));

        // Remove POSIX-specific sources
        lib_sources = lib_sources.iter()
            .cloned()
            .filter(|file| {
                match *file {
                    "port/port_posix.cc" |
                    "util/env_posix.cc" |
                    "env/env_posix.cc" |
                    "env/io_posix.cc" => false,
                    _ => true,
                }
            })
            .collect::<Vec<&'static str>>();

        // Add Windows-specific sources
        lib_sources.push("port/win/port_win.cc");
        lib_sources.push("port/win/env_win.cc");
        lib_sources.push("port/win/env_default.cc");
        lib_sources.push("port/win/win_logger.cc");
        lib_sources.push("port/win/win_thread.cc");
        lib_sources.push("port/win/io_win.cc");
        lib_sources.push("port/win/xpress_win.cc");
    }

    if cfg!(target_env = "msvc") {
        build.flag("-EHsc");
    } else {
        build.flag("-std=c++11");
    }

    for file in lib_sources {
        let file = "rocksdb/".to_string() + file;
        build.file(&file);
    }

    build.file("build_version.cc");
    build.cpp(true);
    build.compile("librocksdb.a");
}

fn build_snappy() {
    let mut build = cc::Build::new();
    build.include("snappy/");
    build.include(".");

    build.define("NDEBUG", Some("1"));
    build.opt_level(3);

    build.warnings(false);
    build.extra_warnings(false);

    if cfg!(target_env = "msvc") {
        build.flag("-EHsc");
    } else {
        build.flag("-std=c++11");
        build.flag("-fPIC");
    }

    build.file("snappy/snappy.cc");
    build.file("snappy/snappy-sinksource.cc");
    build.file("snappy/snappy-c.cc");

    build.cpp(true);
    build.compile("libsnappy.a");
}

fn try_to_find_lib(library: &str) -> bool {
    use std::env;

    let lib_name = match library {
        "librocksdb" => "ROCKSDB",
        "libsnappy" => "SNAPPY",
        "zlib" => "ZLIB",
        _ => "UNKNOWN"
    };

    if let Ok(lib_dir) = env::var(format!("{}_LIB_DIR", lib_name).as_str()) {
        println!("cargo:rustc-link-search=native={}", lib_dir);
        let mode = match env::var_os(format!("{}_STATIC", lib_name).as_str()) {
            Some(_) => "static",
            None => "dylib",
        };
        println!("cargo:rustc-link-lib={0}={1}", mode, lib_name.to_lowercase());
        return true;
    }   

   if probe_library(library).is_ok() {
        true
    } else {
        false
    }
}

fn get_sources(git_path: &str, rev: &str) {
    let mut command = Command::new("git");
    let mut command_result = command
                        .arg("clone")
                        .arg(git_path)
                        .output()
                        .unwrap_or_else(|error| {
                            panic!("Failed to run git command: {}", error);
                        });
    if !command_result.status.success() {   
        panic!("{:?}\n{}\n{}\n", 
            command, 
            String::from_utf8_lossy(&command_result.stdout), 
            String::from_utf8_lossy(&command_result.stderr)
        );
    }

    command = Command::new("git");

    if git_path.contains("snappy") {
        command.current_dir("snappy");
    } else {
        command.current_dir("rocksdb");
    }

    command_result = command
                        .arg("checkout")
                        .arg(rev)
                        .output()
                        .unwrap_or_else(|error| {
                            panic!("Failed to run git command: {}", error);
                        });                              

    if !command_result.status.success() {   
        panic!("{:?}\n{}\n{}\n", 
            command, 
            String::from_utf8_lossy(&command_result.stdout), 
            String::from_utf8_lossy(&command_result.stderr)
        );
    }   
}

fn main() {
    if cfg!(target_os = "linux") {
        assert!(try_to_find_lib("zlib"), "Unable to link with zlib!");
    }
    if !try_to_find_lib("libsnappy") {
        if read_dir("snappy").is_err() {
            get_sources("https://github.com/google/snappy.git", "513df5fb5a2d51146f409141f9eb8736935cc486");
        }
        build_snappy();
    }

    if !try_to_find_lib("librocksdb") {
        if read_dir("rocksdb").is_err() {
            get_sources("https://github.com/facebook/rocksdb.git", "d310e0f33977d4e297bf25a98eef79d1a02513d7");
        }
        build_rocksdb();
    }
}
