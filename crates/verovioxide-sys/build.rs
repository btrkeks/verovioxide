//! Build script for verovioxide-sys.
//!
//! This script compiles the pinned local Verovio C++ library using the `cc` crate.
//!
//! # Local build contract
//!
//! This fork builds only the pinned local fingering implementation. Set
//! `VEROVIO_SOURCE_DIR` to its clean Git checkout. Validation runs before
//! cache lookup. No upstream source or prebuilt binary fallback is available.
//!
//! # Smart Caching
//!
//! To avoid recompiling Verovio (~6 minutes) on every Rust code change, this script
//! implements smart caching:
//!
//! - The compiled library is cached under a build-input fingerprint in
//!   `target/verovio-cache/`
//! - Subsequent builds link to the cached library instead of recompiling
//! - Use `cargo build --features force-rebuild` to force a fresh compilation
//!
//! # Cache Location
//!
//! The cache is stored in the workspace's `target/verovio-cache/` directory to persist
//! across clean builds of individual crates while still being cleaned by `cargo clean`.

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

mod build_source;

const VEROVIO_VERSION: &str = "6.2.1";
const LOCAL_FINGERING_FORK_REVISION: &str = "4c1ef689263c509ab7bbb457046068da2bd19dda";

/// Returns the path to the Verovio cache directory.
///
/// The cache is located at `<workspace_root>/target/verovio-cache/` to ensure it:
/// - Persists across incremental builds
/// - Is cleaned by `cargo clean`
/// - Is shared across all build configurations (debug/release)
fn get_cache_dir() -> PathBuf {
    // Use CARGO_MANIFEST_DIR to find the workspace root reliably.
    // This works regardless of the target directory structure (normal, llvm-cov, etc.)
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    // Navigate up from crates/verovioxide-sys to workspace root
    let workspace_root = manifest_dir
        .parent() // verovioxide-sys -> crates
        .and_then(|p| p.parent()) // crates -> workspace root
        .expect("Failed to find workspace root from CARGO_MANIFEST_DIR");

    workspace_root.join("target").join("verovio-cache")
}

/// Returns the path to the cached static library.
fn get_cached_library_path(cache_dir: &Path) -> PathBuf {
    if cfg!(target_os = "windows") && cfg!(target_env = "msvc") {
        cache_dir.join("verovio.lib")
    } else {
        cache_dir.join("libverovio.a")
    }
}

/// Returns the content-addressed directory for the bundled native library.
///
/// Hashing the build script and verified source inputs prevents an archive
/// compiled from earlier inputs from bypassing a later source fix. The target
/// triple prevents incompatible archives from sharing one cache entry.
fn get_cached_library_dir(source: &Path) -> PathBuf {
    let mut hasher = Sha256::new();
    hasher.update(VEROVIO_VERSION.as_bytes());
    hasher.update(include_bytes!("build.rs"));
    for directory in ["include", "src", "libmei", "tools"] {
        hash_source_tree(source, &source.join(directory), &mut hasher);
    }
    let fingerprint = format!("{:x}", hasher.finalize());
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown-target".to_owned());

    get_cache_dir().join(format!("bundled-{target}-{}", &fingerprint[..16]))
}

fn hash_source_tree(root: &std::path::Path, path: &std::path::Path, hash: &mut Sha256) {
    let mut entries = std::fs::read_dir(path)
        .unwrap_or_else(|error| panic!("read local Verovio source {}: {error}", path.display()))
        .map(|entry| entry.expect("read source entry").path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            hash_source_tree(root, &path, hash);
        } else if path.extension().is_some_and(|extension| {
            matches!(extension.to_str(), Some("h" | "hpp" | "cpp" | "cc" | "c"))
        }) && path.file_name().is_none_or(|name| name != "git_commit.h")
        {
            println!("cargo:rerun-if-changed={}", path.display());
            hash.update(
                path.strip_prefix(root)
                    .expect("source inside root")
                    .to_string_lossy()
                    .as_bytes(),
            );
            hash.update([0]);
            hash.update(std::fs::read(&path).expect("read local Verovio source"));
        }
    }
}

/// Checks if a cached Verovio library exists and should be used.
///
/// Returns `false` if:
/// - The `force-rebuild` feature is enabled
/// - The cached library file doesn't exist
fn should_use_cache(cache_dir: &Path) -> bool {
    // Check for force-rebuild feature via environment variable
    // (cfg! is compile-time, but we need runtime check in build scripts)
    if std::env::var("CARGO_FEATURE_FORCE_REBUILD").is_ok() {
        println!("cargo:warning=force-rebuild feature enabled, recompiling Verovio");
        return false;
    }

    let cached_lib = get_cached_library_path(cache_dir);
    if cached_lib.exists() {
        // The cache hit is the expected steady state; logging it as a
        // cargo:warning would put noise in every downstream build.
        println!("Using cached Verovio library from {}", cached_lib.display());
        true
    } else {
        println!(
            "cargo:warning=No cached Verovio library found at {}, compiling from source",
            cached_lib.display()
        );
        false
    }
}

/// Emits the linker directives to link against the Verovio library.
fn emit_link_directives(search_path: &std::path::Path) {
    println!("cargo:rustc-link-search=native={}", search_path.display());
    println!("cargo:rustc-link-lib=static=verovio");

    // Link the C++ standard library
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=c++");
    } else if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=stdc++");
    } else if cfg!(target_os = "windows") {
        // MSVC links the C++ runtime automatically
        if cfg!(target_env = "gnu") {
            println!("cargo:rustc-link-lib=stdc++");
        }
    }
}

/// Copies the compiled library to the cache directory.
fn cache_compiled_library(out_dir: &Path, cache_dir: &Path) {
    // Create cache directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(cache_dir) {
        println!(
            "cargo:warning=Failed to create cache directory: {}. Caching disabled.",
            e
        );
        return;
    }

    // Determine the library filename based on platform
    let lib_name = if cfg!(target_os = "windows") && cfg!(target_env = "msvc") {
        "verovio.lib"
    } else {
        "libverovio.a"
    };

    let source = out_dir.join(lib_name);
    let dest = cache_dir.join(lib_name);

    if source.exists() {
        match std::fs::copy(&source, &dest) {
            Ok(_) => println!("cargo:warning=Cached Verovio library to {}", dest.display()),
            Err(e) => println!(
                "cargo:warning=Failed to cache library: {}. Future builds may recompile.",
                e
            ),
        }
    } else {
        println!(
            "cargo:warning=Compiled library not found at {}, caching skipped",
            source.display()
        );
    }
}

fn main() {
    // Set up rerun-if-changed directives
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build_source.rs");
    println!("cargo:rerun-if-env-changed=VEROVIO_SOURCE_DIR");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // Check which feature is enabled (use env var since cfg! is compile-time)
    let bundled_enabled = std::env::var("CARGO_FEATURE_BUNDLED").is_ok();
    if !bundled_enabled {
        panic!(
            "This local fingering build requires the bundled feature and the pinned Verovio source; prebuilt archives are unavailable"
        );
    }
    let requested_source = std::env::var_os("VEROVIO_SOURCE_DIR");
    let verovio_dir = build_source::require_pinned_source(
        requested_source.as_deref(),
        LOCAL_FINGERING_FORK_REVISION,
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let verovio_dir = dunce::canonicalize(verovio_dir).expect("normalize validated Verovio path");
    println!("cargo:rerun-if-changed={}", verovio_dir.display());
    let cache_dir = get_cached_library_dir(&verovio_dir);
    if should_use_cache(&cache_dir) {
        emit_link_directives(&cache_dir);
        return;
    }

    std::fs::write(
        out_dir.join("git_commit.h"),
        format!("#define GIT_COMMIT \"{LOCAL_FINGERING_FORK_REVISION}\"\n"),
    )
    .expect("write deterministic Verovio commit header");

    let mut build = cc::Build::new();

    // Configure C++20 standard
    build.cpp(true).std("c++20");

    // Add include directories
    let include_dirs = [
        "include",
        "include/vrv",
        "include/crc",
        "include/midi",
        "include/tuning-library",
        "include/hum",
        "include/json",
        "include/pugi",
        "include/zip",
        "libmei/dist",
        "libmei/addons",
    ];

    build.include(&out_dir);
    for dir in &include_dirs {
        build.include(verovio_dir.join(dir));
    }

    // Platform-specific include for Windows
    if cfg!(target_os = "windows") {
        build.include(verovio_dir.join("include/win32"));
    }

    // Add compiler definitions (matching CMakeLists.txt defaults)
    build.define("NO_DARMS_SUPPORT", None);
    build.define("NO_RUNTIME", None);

    // Set resource directory to a reasonable default
    build.define("RESOURCE_DIR", "\"/usr/local/share/verovio\"");

    // Compiler flags (matching CMakeLists.txt for non-MSVC builds)
    if !cfg!(target_env = "msvc") {
        build
            .flag("-Wall")
            .flag("-W")
            .flag("-pedantic")
            .flag("-Wno-unused-parameter")
            .flag("-Wno-dollar-in-identifier-extension")
            .flag("-Wno-conversion")
            .flag("-Wno-float-conversion")
            .flag("-Wno-missing-braces")
            .flag("-Wno-missing-field-initializers")
            .flag("-Wno-overloaded-virtual")
            .flag("-Wno-shadow")
            .flag("-Wno-sign-conversion")
            .flag("-Wno-trigraphs")
            .flag("-Wno-unknown-pragmas")
            .flag("-Wno-unused-label");
    } else {
        // MSVC-specific settings
        build.flag("/bigobj").flag("/W2").flag("/wd4244");
        build.define("NO_PAE_SUPPORT", None);
        build.define("USE_PAE_OLD_PARSER", None);
    }

    // Collect source files
    let mut sources: Vec<PathBuf> = Vec::new();

    // Main verovio sources (excluding main.cpp)
    for entry in std::fs::read_dir(verovio_dir.join("src")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "cpp")
            && path.file_name().is_some_and(|name| name != "main.cpp")
        {
            sources.push(path);
        }
    }

    // Humdrum sources
    if let Ok(entries) = std::fs::read_dir(verovio_dir.join("src/hum")) {
        for entry in entries {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "cpp") {
                sources.push(path);
            }
        }
    }

    // MIDI sources
    for entry in std::fs::read_dir(verovio_dir.join("src/midi")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "cpp") {
            sources.push(path);
        }
    }

    // CRC sources
    for entry in std::fs::read_dir(verovio_dir.join("src/crc")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "cpp") {
            sources.push(path);
        }
    }

    // JSON source (note: .cc extension)
    sources.push(verovio_dir.join("src/json/jsonxx.cc"));

    // pugixml source
    sources.push(verovio_dir.join("src/pugi/pugixml.cpp"));

    // libmei dist sources
    for entry in std::fs::read_dir(verovio_dir.join("libmei/dist")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "cpp") {
            sources.push(path);
        }
    }

    // libmei addons sources
    for entry in std::fs::read_dir(verovio_dir.join("libmei/addons")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "cpp") {
            sources.push(path);
        }
    }

    // C wrapper
    sources.push(verovio_dir.join("tools/c_wrapper.cpp"));

    // Add all source files to the build
    for source in &sources {
        build.file(source);
    }

    // Compile the library
    build.compile("verovio");

    // Cache the compiled library for future builds
    cache_compiled_library(&out_dir, &cache_dir);

    // Emit link directives (cc::Build::compile already sets up linking,
    // but we emit them explicitly for consistency)
    emit_link_directives(&out_dir);
}
