use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    println!("cargo:rerun-if-changed={crate_dir}/src/ffi.rs");

    let output_file = format!("{crate_dir}/include/stl2thumbnail.h");
    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_namespace("s2t")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(output_file);
}
