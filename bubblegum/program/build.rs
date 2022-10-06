use std::{path::Path, process::Command};

// This build script runs `./download-compression-programs.sh` if either of the paths
// below does not exist. Please note that running the bash script with replace both
// files anyway. If both are already present, pretty much nothing happens. They are only
// required for running the tests, but there's no way currently to have a `build.rs`
// action for tests alone.
fn main() {
    // The build script's working folder is always that of the containing package.
    let spl_compression_so_path =
        Path::new("../../target/deploy/GRoLLzvxpxxu2PGNJMMeZPyMxjAUH9pKqxGXV9DGiceU.so");
    let spl_wrapper_so_path =
        Path::new("../../target/deploy/WRAPYChf58WFCnyjXKJHtrPgzKXgHp6MD9aVDqJBbGh.so");

    if !spl_compression_so_path.exists() || !spl_wrapper_so_path.exists() {
        Command::new("./download-compression-programs.sh")
            .output()
            .expect("failed to execute download compression programs script");
    }

    // The `build.rs` file be default is not re-run unless some files in the current crate
    // change, but the compression libraries reside outside, so we need to issue the prints
    // below (which have a special significance as part of the semantics around `buils.rs`
    // files) to effectively tell the script to run if any of them are missing (or changed).
    println!(
        "cargo:rerun-if-changed={}",
        spl_compression_so_path.display()
    );
    println!("cargo:rerun-if-changed={}", spl_wrapper_so_path.display());
}
