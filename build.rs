use anyhow::Result;
use std::io::Write;
use std::{env, fs::File, path::PathBuf};

use get_root_flags::get_root_flags;

fn main() -> Result<()> {
    compile_ntuple_writer()
}

fn compile_ntuple_writer() -> Result<()> {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let bindings = bindgen::Builder::default()
        .header("src/ntuple.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("ntuple_create_reader")
        .allowlist_function("ntuple_read_event")
        .allowlist_function("ntuple_num_events")
        .allowlist_function("ntuple_delete_reader")
        .allowlist_function("ntuple_create_writer")
        .allowlist_function("ntuple_write_event")
        .allowlist_function("ntuple_delete_writer")
        .newtype_enum("ReadStatus")
        .newtype_enum("WriteResult")
        .generate()
        .expect("Failed to generate ntuple writer bindings");

    bindings
        .write_to_file(out_path.join("ntuple.rs"))
        .expect("Failed to write ntuple bindings!");

    println!("cargo:rerun-if-changed=src/ntupleevent.h");
    println!("cargo:rerun-if-changed=src/ntuplereader.h");
    println!("cargo:rerun-if-changed=src/ntuplewriter.h");
    println!("cargo:rerun-if-changed=src/root_interface.hh");
    println!("cargo:rerun-if-changed=src/ntuplereader.cc");
    println!("cargo:rerun-if-changed=src/ntuplewriter.cc");
    println!("cargo:rerun-if-changed=src/root_interface.cc");
    let mut cc_cmd = cc::Build::new();
    cc_cmd.cpp(true).files([
        "src/ntuplereader.cc",
        "src/ntuplewriter.cc",
        "src/root_interface.cc",
    ]);

    for flag in get_root_flags("--cflags")? {
        cc_cmd.flag(&flag);
    }

    cc_cmd.compile("ntuplewriter");

    let root_linker_flags = get_root_flags("--libs")?;
    let linker_flags =
        Vec::from_iter(root_linker_flags.iter().map(|f| format!(r#"r"{f}""#)));
    let mut flag_out = File::create(out_path.join("flags.rs"))?;

    writeln!(
        flag_out,
        "pub const ROOT_LINKER_FLAGS: [&str; {}] = [{}];",
        linker_flags.len(),
        linker_flags.join(", ")
    )?;

    for flag in root_linker_flags {
        println!("cargo:rustc-link-arg={flag}");
    }

    Ok(())
}
