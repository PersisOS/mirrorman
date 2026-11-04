use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub action: String,
    pub package: String,
    pub version_info: String,
}

pub fn parse_pacman_log() -> Vec<LogEntry> {
    parse_system_log()
}

pub fn parse_system_log() -> Vec<LogEntry> {
    if let Ok(c) = fs::read_to_string("/var/log/dpkg.log") {
        let entries = parse_dpkg_log_str(&c);
        if !entries.is_empty() {
            return entries;
        }
    }
    if let Ok(c) = fs::read_to_string("/var/log/pacman.log") {
        return parse_pacman_log_str(&c);
    }
    Vec::new()
}

pub fn parse_log_str(content: &str) -> Vec<LogEntry> {
    if content.contains("[ALPM]") {
        parse_pacman_log_str(content)
    } else {
        parse_dpkg_log_str(content)
    }
}

pub fn parse_dpkg_log_str(content: &str) -> Vec<LogEntry> {
    let mut entries = Vec::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let date = parts[0];
        let time = parts[1];
        let timestamp = format!("{date} {time}");

        let (action, pkg, ver) = if parts[2] == "status" && parts.len() >= 5 {
            let state = parts[3];
            let raw_pkg = parts[4];
            let ver = parts.get(5).copied().unwrap_or("");
            let act = match state {
                "installed" => "Installed",
                "removed" => "Removed",
                _ => continue,
            };
            (act, raw_pkg, ver)
        } else if parts[2] == "upgrade" && parts.len() >= 6 {
            let raw_pkg = parts[3];
            let old_ver = parts[4];
            let new_ver = parts[5];
            let ver_info = format!("{old_ver} -> {new_ver}");
            entries.push(LogEntry {
                timestamp,
                action: "Upgraded".to_string(),
                package: raw_pkg.split(':').next().unwrap_or(raw_pkg).to_string(),
                version_info: ver_info,
            });
            continue;
        } else if parts[2] == "install" && parts.len() >= 5 {
            ("Installed", parts[3], parts.get(4).copied().unwrap_or(""))
        } else if parts[2] == "remove" && parts.len() >= 5 {
            ("Removed", parts[3], parts.get(4).copied().unwrap_or(""))
        } else {
            continue;
        };

        let clean_pkg = pkg.split(':').next().unwrap_or(pkg).to_string();
        let clean_ver = ver.trim_matches(|c| c == '(' || c == ')').to_string();

        entries.push(LogEntry {
            timestamp,
            action: action.to_string(),
            package: clean_pkg,
            version_info: clean_ver,
        });
    }

    entries.reverse();
    entries
}

pub fn parse_pacman_log_str(content: &str) -> Vec<LogEntry> {
    let mut entries = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('[') {
            continue;
        }

        if let Some(close_bracket) = line.find(']') {
            let timestamp = &line[1..close_bracket];
            let rest = line[close_bracket + 1..].trim();

            if rest.contains("[ALPM]") {
                let msg = rest.replace("[ALPM]", "").trim().to_string();
                if let Some((action, detail)) = msg.split_once(' ') {
                    let action_str = match action {
                        "installed" => "Installed",
                        "upgraded" => "Upgraded",
                        "removed" => "Removed",
                        "reinstalled" => "Reinstalled",
                        "downgraded" => "Downgraded",
                        _ => continue,
                    };

                    let (pkg, ver) = if let Some((p, v)) = detail.split_once(' ') {
                        (p.to_string(), v.trim_matches(|c| c == '(' || c == ')').to_string())
                    } else {
                        (detail.to_string(), String::new())
                    };

                    entries.push(LogEntry {
                        timestamp: timestamp.to_string(),
                        action: action_str.to_string(),
                        package: pkg,
                        version_info: ver,
                    });
                }
            }
        }
    }

    entries.reverse();
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_log_str_pacman() {
        let sample = "\
[2026-07-24T00:10:00+0330] [ALPM] transaction started
[2026-07-24T00:10:01+0330] [ALPM] installed ripgrep (14.1.0-1)
[2026-07-24T00:10:02+0330] [ALPM] upgraded gtk4 (4.14.0 -> 4.16.0)
[2026-07-24T00:10:03+0330] [ALPM] removed nano (7.2-1)
[2026-07-24T00:10:04+0330] [ALPM] transaction completed
";

        let entries = parse_log_str(sample);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].package, "nano");
        assert_eq!(entries[0].action, "Removed");
        assert_eq!(entries[1].package, "gtk4");
        assert_eq!(entries[1].action, "Upgraded");
        assert_eq!(entries[2].package, "ripgrep");
        assert_eq!(entries[2].action, "Installed");
    }

    #[test]
    fn test_parse_log_str_dpkg() {
        let sample = "\
2026-11-04 10:15:30 install ripgrep:amd64 <none> 14.1.0-1
2026-11-04 10:15:31 status installed ripgrep:amd64 14.1.0-1
2026-11-04 10:15:32 upgrade gtk4:amd64 4.14.0 4.16.0
2026-11-04 10:15:34 remove nano:amd64 7.2-1 <none>
";

        let entries = parse_dpkg_log_str(sample);
        assert!(!entries.is_empty());
        assert!(entries.iter().any(|e| e.package == "ripgrep" && e.action == "Installed"));
        assert!(entries.iter().any(|e| e.package == "gtk4" && e.action == "Upgraded"));
        assert!(entries.iter().any(|e| e.package == "nano" && e.action == "Removed"));
    }
}
