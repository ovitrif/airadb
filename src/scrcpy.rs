use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

use crate::command_path::resolve_program;

#[derive(Debug, Clone)]
pub struct Scrcpy {
    path: PathBuf,
}

impl Scrcpy {
    pub fn resolve(override_path: Option<PathBuf>, skip_check: bool) -> Result<Self> {
        let path = if skip_check {
            override_path.unwrap_or_else(|| PathBuf::from("scrcpy"))
        } else {
            resolve_program("scrcpy", override_path)?
        };

        Ok(Self { path })
    }

    pub fn launch(&self, serial: &str) -> Result<()> {
        let status = Command::new(&self.path)
            .args(default_args(serial))
            .status()
            .with_context(|| format!("failed to run {}", self.path.display()))?;

        if status.success() {
            return Ok(());
        }

        bail!("scrcpy exited with status {status}");
    }

    pub fn launch_background(&self, serial: &str) -> Result<u32> {
        let child = Command::new(&self.path)
            .args(default_args(serial))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("failed to run {}", self.path.display()))?;

        Ok(child.id())
    }
}

pub fn default_args(serial: &str) -> Vec<OsString> {
    vec![
        OsString::from("-s"),
        OsString::from(serial),
        OsString::from("--no-audio"),
        OsString::from("--stay-awake"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_raycast_equivalent_scrcpy_args() {
        let args = default_args("192.168.1.23:40233");
        let args: Vec<_> = args
            .iter()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect();

        assert_eq!(
            args,
            vec!["-s", "192.168.1.23:40233", "--no-audio", "--stay-awake",]
        );
    }
}
