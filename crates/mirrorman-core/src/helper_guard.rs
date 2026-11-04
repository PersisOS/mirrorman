//! Whitelist validation for privileged helper operations in Debian GNU/Linux.
//!
//! Shared by the root `mirrorman-helper` D-Bus service and the unprivileged
//! client so both sides enforce the same allow-lists. Nothing here runs any
//! command; it only decides whether a proposed invocation is safe enough to
//! forward to a root context.

/// Allowed APT subcommands and flags.
const ALLOWED_APT_OPS: &[&str] = &[
    "update", "clean", "autoclean", "autoremove", "install", "check",
];

const ALLOWED_APT_FLAGS: &[&str] = &[
    "-y", "--assume-yes", "-q", "--quiet", "--dry-run", "-f", "--fix-broken",
    "--allow-releaseinfo-change",
];

/// Validates arguments passed to apt / apt-get.
pub fn validate_apt_args(args: &[String]) -> Result<(), String> {
    let Some(first) = args.first() else {
        return Err("apt requires at least one operation flag".to_string());
    };
    if !ALLOWED_APT_OPS.contains(&first.as_str()) {
        return Err(format!("apt operation '{first}' is not allowed"));
    }
    for a in &args[1..] {
        if a.starts_with('-') {
            if !ALLOWED_APT_FLAGS.contains(&a.as_str()) {
                return Err(format!("apt flag '{a}' is not allowed"));
            }
            continue;
        }
        if !is_safe_package_value(a) {
            return Err(format!("apt argument '{a}' is not allowed"));
        }
    }
    Ok(())
}

/// Package spec or safe path.
fn is_safe_package_value(s: &str) -> bool {
    if s.is_empty() || s.len() >= 2048 {
        return false;
    }
    if s.starts_with('/') {
        return !s.contains("..");
    }
    s.chars().all(|c| c.is_ascii_alphanumeric() || "-._+:=~".contains(c))
}

const ALLOWED_CP_SOURCES: &[&str] = &[
    "/etc/apt/sources.list",
    "/etc/apt/sources.list.d/debian.sources",
    "/etc/apt/sources.list.d/mirrors.list",
    "/etc/apt/apt.conf.d/99mirrorman.conf",
];

/// The `cp` command is allowed to back up and restore APT configuration files.
pub fn validate_cp_args(args: &[String]) -> Result<(), String> {
    if args.len() != 2 {
        return Err("cp requires exactly two arguments".to_string());
    }
    let src = &args[0];
    if !ALLOWED_CP_SOURCES.contains(&src.as_str()) {
        return Err(format!("cp source '{src}' is not allowed"));
    }
    let dest = &args[1];
    let is_valid_dest = dest.starts_with("/etc/apt/")
        && !dest.contains("..")
        && dest.len() < 2048;

    if !is_valid_dest {
        return Err("cp destination must be a valid path under /etc/apt/".to_string());
    }
    Ok(())
}

/// Validate a `RunCommand` request.
pub fn validate_run_command(command: &str, args: &[String]) -> Result<(), String> {
    match command {
        "cp" => validate_cp_args(args),
        _ => Err(format!("command '{command}' is not whitelisted")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &[&str]) -> Vec<String> {
        s.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn apt_update_allowed() {
        assert!(validate_apt_args(&v(&["update"])).is_ok());
        assert!(validate_apt_args(&v(&["clean"])).is_ok());
        assert!(validate_apt_args(&v(&["autoclean"])).is_ok());
        assert!(validate_apt_args(&v(&["autoremove", "-y"])).is_ok());
    }

    #[test]
    fn apt_dangerous_rejected() {
        assert!(validate_apt_args(&v(&[])).is_err());
        assert!(validate_apt_args(&v(&["purge", "*"])).is_err());
        assert!(validate_apt_args(&v(&["install", "curl; rm -rf /"])).is_err());
    }

    #[test]
    fn cp_apt_sources_allowed() {
        assert!(validate_cp_args(&v(&[
            "/etc/apt/sources.list",
            "/etc/apt/sources.list.backup.20261104",
        ])).is_ok());
        assert!(validate_cp_args(&v(&[
            "/etc/apt/sources.list.d/debian.sources",
            "/etc/apt/sources.list.d/debian.sources.backup",
        ])).is_ok());
    }

    #[test]
    fn cp_abuse_rejected() {
        assert!(validate_cp_args(&v(&["/etc/apt/sources.list", "/tmp/passwd"])).is_err());
        assert!(validate_cp_args(&v(&["/etc/apt/sources.list", "/etc/apt/../shadow"])).is_err());
        assert!(validate_cp_args(&v(&["/etc/shadow", "/etc/apt/shadow.bak"])).is_err());
    }

    #[test]
    fn run_command_whitelist() {
        assert!(validate_run_command("cp", &v(&["/etc/apt/sources.list", "/etc/apt/sources.list.bak"])).is_ok());
        assert!(validate_run_command("apt", &v(&["update"])).is_err());
        assert!(validate_run_command("rm", &v(&["-rf", "/"])).is_err());
    }
}
