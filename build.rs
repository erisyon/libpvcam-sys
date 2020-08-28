extern crate bindgen;

use std::env;
use std::path::PathBuf;

const SDK_PATH_KEY: &str = "PVCAM_SDK_PATH";

fn sdk_path() -> String {
    // if this is not set the build will panic
    let path = match env::var(SDK_PATH_KEY) {
        Ok(v) => v,
        Err(_) => panic!("{} must be set", SDK_PATH_KEY),
    };

    path
}

/**
 * TODO: expand the logic here to incorporate other OSes and architectures
 * see: https://doc.rust-lang.org/reference/conditional-compilation.html#conditional-compilation
 */
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn libpvcam_search_path() -> String {
    format!("{}/library/x86_64", sdk_path())
}

fn header_include_path() -> String {
    format!("{}/include", sdk_path())
}

fn main() {
    // tell cargo to tell rust c to link to pvcam
    println!("cargo:rustc-link-lib=pvcam");

    // tell cargo to tell rust c where to search for pvcam
    println!("cargo:rustc-link-search={}", libpvcam_search_path());

    // Tell cargo to invalidate the build if the header files change
    println!("cargo:rerun-if-changed={}/master.h", header_include_path());
    println!("cargo:rerun-if-changed={}/pvcam.h", header_include_path());

    let bindings = bindgen::Builder::default()
        // generate bindings for these headers
        .header(format!("{}/master.h", header_include_path()))
        .header(format!("{}/pvcam.h", header_include_path()))
        // BLACKLIST: pl_cam_open; REASON: camera_name variable incorrectly marked as mutable
        .blacklist_function("pl_cam_open")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
