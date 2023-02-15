use std::env;
use std::{path::Path, process::Command};
// This build script runs `./download-compression-programs.sh` if either of the paths
// below does not exist. Please note that running the bash script with replace both
// files anyway. If both are already present, pretty much nothing happens. They are only
// required for running the tests, but there's no way currently to have a `build.rs`
// action for tests alone.
fn main() {
    let ci = env::var("CI_PUBLISH");
    if ci.unwrap_or_else(|_| "false".to_string()) == *"false" {
        // The build script's working folder is always that of the containing package.
        let spl_compression_so_path = Path::new("../../test-programs/spl_account_compression.so");
        let spl_wrapper_so_path = Path::new("../../test-programs/spl_noop.so");
        let mpl_tm_so_path = Path::new("../../test-programs/mpl_token_metadata.so");

        if !spl_compression_so_path.exists() || !spl_wrapper_so_path.exists() {
            Command::new("./download-compression-programs.sh")
                .output()
                .expect("failed to execute build TM script");
        }

        if !mpl_tm_so_path.exists() {
            Command::new("./build-token-metadata.sh")
                .output()
                .expect("failed to execute download compression programs script");
        }

        let paths = [spl_compression_so_path, spl_wrapper_so_path, mpl_tm_so_path];

        // The `build.rs` file be default is not re-run unless some files in the current crate
        // change, but the compression libraries reside outside, so we need to issue the prints
        // below (which have a special significance as part of the semantics around `buils.rs`
        // files) to effectively tell the script to run if any of them are missing (or changed).
        for path in paths {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
