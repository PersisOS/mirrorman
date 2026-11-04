use crate::helper_guard::{validate_apt_args, validate_run_command};
use zbus::blocking::Connection;

pub struct HelperClient;

const MAX_CONTENT_LEN: usize = 1_000_000;

fn call_helper(
    method: &str,
    body: &(impl serde::Serialize + zbus::zvariant::DynamicType),
) -> Result<zbus::Message, zbus::Error> {
    let conn = Connection::system()?;
    conn.call_method(
        Some("com.parchlinux.mirrorman.Helper"),
        "/com/parchlinux/mirrorman/Helper",
        Some("com.parchlinux.mirrorman.Helper"),
        method,
        body,
    )
}

impl HelperClient {
    pub fn save_sources_list(content: &str) -> Result<(), String> {
        if content.len() > MAX_CONTENT_LEN {
            return Err("sources.list content too large".to_string());
        }
        if let Ok(reply) = call_helper("SaveMirrorlist", &(content,)) {
            if let Ok(true) = reply.body().deserialize::<bool>() {
                return Ok(());
            }
        }
        Self::fallback_save_sources_list(content)
    }

    pub fn save_deb822_sources(content: &str) -> Result<(), String> {
        if content.len() > MAX_CONTENT_LEN {
            return Err("debian.sources content too large".to_string());
        }
        if let Ok(reply) = call_helper("SaveDeb822Sources", &(content,)) {
            if let Ok(true) = reply.body().deserialize::<bool>() {
                return Ok(());
            }
        }
        Self::fallback_save_deb822_sources(content)
    }

    pub fn save_mirrorlist(content: &str) -> Result<(), String> {
        Self::save_sources_list(content)
    }

    pub fn save_apt_conf(content: &str) -> Result<(), String> {
        if content.len() > MAX_CONTENT_LEN {
            return Err("apt.conf content too large".to_string());
        }
        if let Ok(reply) = call_helper("SavePacmanConf", &(content,)) {
            if let Ok(true) = reply.body().deserialize::<bool>() {
                return Ok(());
            }
        }
        Self::fallback_save_apt_conf(content)
    }

    pub fn save_pacman_conf(content: &str) -> Result<(), String> {
        Self::save_apt_conf(content)
    }

    pub fn run_apt(args: &[&str]) -> Result<(bool, String, String), String> {
        let vec_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        validate_apt_args(&vec_args)?;
        if let Ok(reply) = call_helper("RunApt", &(vec_args.clone(),)) {
            if let Ok(tuple) = reply.body().deserialize::<(bool, String, String)>() {
                return Ok(tuple);
            }
        }
        if let Ok(reply) = call_helper("RunPacman", &(vec_args,)) {
            if let Ok(tuple) = reply.body().deserialize::<(bool, String, String)>() {
                return Ok(tuple);
            }
        }
        Self::fallback_run_apt(args)
    }

    pub fn run_pacman(args: &[&str]) -> Result<(bool, String, String), String> {
        Self::run_apt(args)
    }

    pub fn run_command(command: &str, args: &[&str]) -> Result<(bool, String, String), String> {
        let vec_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        validate_run_command(command, &vec_args)?;
        if let Ok(reply) = call_helper("RunCommand", &(command, vec_args)) {
            if let Ok(tuple) = reply.body().deserialize::<(bool, String, String)>() {
                return Ok(tuple);
            }
        }
        Self::fallback_run_command(command, args)
    }

    fn fallback_save_sources_list(content: &str) -> Result<(), String> {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new()
            .map_err(|e| format!("Failed to create temp file: {e}"))?;
        tmp.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write sources.list: {e}"))?;
        let temp_path = tmp.path().to_str()
            .ok_or("Temp path contains invalid UTF-8")?;

        let status = std::process::Command::new("pkexec")
            .args(["cp", temp_path, "/etc/apt/sources.list"])
            .status()
            .map_err(|e| format!("pkexec failed: {e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err("pkexec failed to save sources file".to_string())
        }
    }

    fn fallback_save_deb822_sources(content: &str) -> Result<(), String> {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new()
            .map_err(|e| format!("Failed to create temp file: {e}"))?;
        tmp.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write deb822 sources: {e}"))?;
        let temp_path = tmp.path().to_str()
            .ok_or("Temp path contains invalid UTF-8")?;

        let status = std::process::Command::new("pkexec")
            .args(["cp", temp_path, "/etc/apt/sources.list.d/debian.sources"])
            .status()
            .map_err(|e| format!("pkexec failed: {e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err("pkexec failed to save deb822 sources".to_string())
        }
    }

    fn fallback_save_apt_conf(content: &str) -> Result<(), String> {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new()
            .map_err(|e| format!("Failed to create temp file: {e}"))?;
        tmp.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write config: {e}"))?;
        let temp_path = tmp.path().to_str()
            .ok_or("Temp path contains invalid UTF-8")?;

        let _ = std::fs::create_dir_all("/etc/apt/apt.conf.d");
        let status = std::process::Command::new("pkexec")
            .args(["cp", temp_path, "/etc/apt/apt.conf.d/99mirrorman.conf"])
            .status()
            .map_err(|e| format!("pkexec failed: {e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err("pkexec failed to save configuration".to_string())
        }
    }

    fn fallback_run_apt(args: &[&str]) -> Result<(bool, String, String), String> {
        let cmd = if std::path::Path::new("/usr/bin/apt-get").exists() {
            "apt-get"
        } else {
            "apt"
        };

        let output = std::process::Command::new("pkexec")
            .arg(cmd)
            .args(args)
            .output()
            .map_err(|e| format!("Failed to execute pkexec {cmd}: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Ok((output.status.success(), stdout, stderr))
    }

    fn fallback_run_command(
        command: &str,
        args: &[&str],
    ) -> Result<(bool, String, String), String> {
        validate_run_command(command, &args.iter().map(|s| s.to_string()).collect::<Vec<_>>())?;
        let path = match command {
            "cp" => "cp",
            _ => return Err(format!("command '{command}' is not whitelisted")),
        };
        let output = std::process::Command::new("pkexec")
            .arg(path)
            .args(args)
            .output()
            .map_err(|e| format!("Failed to execute pkexec {command}: {e}"))?;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Ok((output.status.success(), stdout, stderr))
    }
}
