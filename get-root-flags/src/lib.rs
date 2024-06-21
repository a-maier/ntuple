use std::process::Command;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to run {cmd}")]
    Cmd{cmd: String, source: std::io::Error},
    #[error("'{cmd}' failed without error message")]
    RunNoErr{cmd: String},
    #[error("'{cmd}' failed: {err}")]
    Run{cmd: String, err: String},
}

/// Get flags returned from `root-config` with the given argument flags
pub fn get_root_flags(flags: &str) -> Result<Vec<String>, Error> {
    use Error::*;

    const CFG_CMD: &str = "root-config";

    let output = Command::new(CFG_CMD)
        .arg(flags)
        .output()
        .map_err(|source| {
            let cmd = format!("{CFG_CMD} {flags}");
            Cmd{cmd, source}
        })?;
    if !output.status.success() {
        let cmd = format!("{CFG_CMD} {flags}");
        if output.stderr.is_empty() {
            return Err(RunNoErr{cmd});
        } else {
            let err = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Run{cmd, err});
        }
    }
    let args = String::from_utf8_lossy(&output.stdout);
    Ok(args.split_whitespace().map(|arg| arg.to_owned()).collect())
}
