use mirrorman_core::helper_client::HelperClient;
use mirrorman_core::helper_guard::{validate_apt_args, validate_run_command};

#[test]
fn test_helper_client_invalid_command_whitelist() {
    let result = HelperClient::run_command("invalid_custom_binary", &["--test"]);
    assert!(result.is_err(), "non-whitelisted command must be rejected client-side");
}

#[test]
fn test_helper_client_shell_commands_rejected() {
    let result = HelperClient::run_command("bash", &["-c", "rm -rf /"]);
    assert!(result.is_err(), "bash must never be executable via run_command");
}

#[test]
fn test_helper_client_curl_rejected() {
    let result = HelperClient::run_command("curl", &["-O", "https://example.com/test"]);
    assert!(result.is_err(), "curl must never be executable via run_command");
}

#[test]
fn test_guard_apt_args() {
    assert!(validate_apt_args(&["update".to_string()]).is_ok());
    assert!(validate_apt_args(&["clean".to_string()]).is_ok());
    assert!(validate_apt_args(&["autoremove".to_string(), "-y".to_string()]).is_ok());
    assert!(validate_apt_args(&["purge".to_string()]).is_err());
    assert!(validate_apt_args(&["install".to_string(), "foo; rm -rf /".to_string()]).is_err());
}

#[test]
fn test_guard_run_command() {
    assert!(validate_run_command(
        "cp",
        &["/etc/apt/sources.list".to_string(), "/etc/apt/sources.list.backup".to_string()]
    )
    .is_ok());
    assert!(validate_run_command("bash", &["-c".to_string(), "true".to_string()]).is_err());
    assert!(validate_run_command("apt", &["update".to_string()]).is_err());
}
