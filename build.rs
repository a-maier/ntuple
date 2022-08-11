use std::io::Write;
use std::{path::PathBuf, env, fs::File};
use anyhow::Result;

fn main() -> Result<()> {
    compile_ntuple_writer()
}

fn compile_ntuple_writer() -> Result<()> {
    let bindings = bindgen::Builder::default()
        .header("src/ntuplewriter.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .allowlist_function("ntuple_create_writer")
        .allowlist_function("ntuple_write_event")
        .allowlist_function("ntuple_delete_writer")
        .newtype_enum("WriteResult")
        .generate()
        .expect("Failed to generate ntuple writer bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("ntuplewriter.rs"))
        .expect("Failed to write ntuple writer bindings!");

    println!("cargo:rerun-if-changed=src/ntuplewriter.h");
    println!("cargo:rerun-if-changed=src/ntuplewriter.cc");
    let mut cc_cmd = cc::Build::new();
    cc_cmd
        .cpp(true)
        .file("src/ntuplewriter.cc");

    for flag in get_root_flags("--cflags")? {
        cc_cmd.flag(&flag);
    }

    cc_cmd.compile("ntuplewriter");

    let linker_flags =  Vec::from_iter(
        get_root_flags("--libs")?.into_iter().map(|f| format!(r#"r"{f}""#))
    );
    let mut flag_out = File::create(out_path.join("flags.rs"))?;

    writeln!(
        flag_out,
        "pub const ROOT_LINKER_FLAGS: [&str; {}] = [{}];",
        linker_flags.len(),
        linker_flags.join(", ")
    )?;
    Ok(())
}

fn get_root_flags(flags: &str) -> Result<Vec<String>> {
    use std::{process::Command, str::from_utf8};
    use anyhow::{anyhow, Context};

    const CFG_CMD: &str = "root-config";

    let cmd = format!("{CFG_CMD} {flags}");
    let output = Command::new(CFG_CMD).arg(flags).output().with_context(
        || format!("Failed to run `{cmd}`")
    )?;
    if !output.status.success() {
        if output.stderr.is_empty() {
            return Err(
                anyhow!("{CFG_CMD} {flags} failed without error messages")
            );
        } else {
            return Err(anyhow!(
                "{CFG_CMD} {flags} failed: {}",
                from_utf8(&output.stderr).unwrap()
            ));
        }
    }
    let args = from_utf8(&output.stdout).with_context(
        || format!("Failed to convert `{cmd}` output to utf8")
    )?;
    Ok(args.split_whitespace().map(|arg| arg.to_owned()).collect())
}
