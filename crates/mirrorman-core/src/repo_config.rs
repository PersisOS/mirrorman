use std::collections::HashMap;

const SOURCES_LIST: &str = "/etc/apt/sources.list";

pub struct RepoConfig {
    pub sources_list: &'static str,
    pub standard_repos: Vec<String>,
    pub third_party_repos: Vec<String>,
    pub custom_repos: Vec<String>,
    pub repositories: HashMap<String, bool>,
}

impl Default for RepoConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl RepoConfig {
    pub fn new() -> Self {
        let standard_repos = vec![
            "main".to_string(),
            "contrib".to_string(),
            "non-free".to_string(),
            "non-free-firmware".to_string(),
        ];
        let third_party_repos = vec![
            "trixie-backports".to_string(),
            "persis-extra".to_string(),
            "liquorix".to_string(),
        ];

        let mut repositories = HashMap::new();
        for repo in standard_repos.iter().chain(third_party_repos.iter()) {
            repositories.insert(repo.clone(), true);
        }

        let mut config = Self {
            sources_list: SOURCES_LIST,
            standard_repos,
            third_party_repos,
            custom_repos: Vec::new(),
            repositories,
        };

        config.load_sources_list();
        config
    }

    fn load_sources_list(&mut self) {
        let content = match std::fs::read_to_string(self.sources_list) {
            Ok(c) => c,
            Err(_) => return,
        };

        for repo in &self.standard_repos {
            let mut enabled = false;
            for line in content.lines() {
                let stripped = line.trim();
                if stripped.starts_with("deb ") && stripped.contains(repo) {
                    enabled = true;
                    break;
                }
            }
            self.repositories.insert(repo.clone(), enabled);
        }

        for repo in &self.third_party_repos {
            let mut enabled = false;
            for line in content.lines() {
                let stripped = line.trim();
                if stripped.starts_with("deb ") && stripped.contains(repo) {
                    enabled = true;
                    break;
                }
            }
            self.repositories.insert(repo.clone(), enabled);
        }
    }

    pub fn toggle_repo_in_config(
        &mut self,
        repo_name: &str,
        enable: bool,
        is_third_party: bool,
    ) -> Result<(), String> {
        let config_text = std::fs::read_to_string(self.sources_list).unwrap_or_default();
        let mut lines: Vec<String> = Vec::new();

        if self.standard_repos.contains(&repo_name.to_string()) {
            for line in config_text.lines() {
                if line.contains("deb ") || line.contains("deb-src ") {
                    let mut parts: Vec<&str> = line.split_whitespace().collect();
                    if enable {
                        if !parts.contains(&repo_name) {
                            parts.push(repo_name);
                        }
                    } else {
                        parts.retain(|&p| p != repo_name);
                    }
                    lines.push(parts.join(" "));
                } else {
                    lines.push(line.to_string());
                }
            }
        } else if is_third_party {
            let snippet = get_third_party_section(repo_name).unwrap_or_default();
            let mut found = false;
            for line in config_text.lines() {
                if line.contains(repo_name) {
                    found = true;
                    if enable {
                        lines.push(line.trim_start_matches('#').trim().to_string());
                    } else {
                        if !line.starts_with('#') {
                            lines.push(format!("#{line}"));
                        } else {
                            lines.push(line.to_string());
                        }
                    }
                } else {
                    lines.push(line.to_string());
                }
            }
            if !found && enable && !snippet.is_empty() {
                lines.push(format!("\n# Third-party repo {repo_name}"));
                lines.push(snippet);
            }
        }

        let modified = lines.join("\n");
        crate::helper_client::HelperClient::save_sources_list(&modified)?;
        self.repositories.insert(repo_name.to_string(), enable);
        Ok(())
    }

    pub fn add_repository(&mut self, repo_name: &str, repo_url: &str, _siglevel: &str) -> Result<(), String> {
        if repo_name.is_empty() || repo_url.is_empty() {
            return Err("Repository name and URL are required".to_string());
        }
        if self.repositories.contains_key(repo_name) {
            return Err(format!("Repository already exists: '{repo_name}'"));
        }

        let config_text = std::fs::read_to_string(self.sources_list).unwrap_or_default();
        let line = if repo_url.starts_with("deb ") {
            repo_url.to_string()
        } else {
            format!("deb {repo_url} trixie main")
        };
        let modified = format!("{config_text}\n# Custom repository: {repo_name}\n{line}\n");
        crate::helper_client::HelperClient::save_sources_list(&modified)?;

        self.repositories.insert(repo_name.to_string(), true);
        self.custom_repos.push(repo_name.to_string());
        Ok(())
    }

    pub fn remove_repository(&mut self, repo_name: &str) -> Result<(), String> {
        if !self.custom_repos.iter().any(|r| r == repo_name) {
            return Err(format!("'{repo_name}' is not a custom repository"));
        }

        let config_text = std::fs::read_to_string(self.sources_list).unwrap_or_default();
        let mut lines = Vec::new();
        let mut skip_next = false;
        for line in config_text.lines() {
            if line.contains(repo_name) {
                skip_next = true;
                continue;
            }
            if skip_next && line.starts_with("deb ") {
                skip_next = false;
                continue;
            }
            skip_next = false;
            lines.push(line);
        }
        let modified = lines.join("\n");
        crate::helper_client::HelperClient::save_sources_list(&modified)?;

        self.repositories.remove(repo_name);
        self.custom_repos.retain(|r| r != repo_name);
        Ok(())
    }

    pub fn enable_third_party(&self, repo_name: &str) -> Result<(), String> {
        match repo_name {
            "trixie-backports" => {
                let deb_line = "deb https://deb.debian.org/debian trixie-backports main contrib non-free non-free-firmware";
                let config_text = std::fs::read_to_string(self.sources_list).unwrap_or_default();
                if !config_text.contains("trixie-backports") {
                    let modified = format!("{config_text}\n{deb_line}\n");
                    crate::helper_client::HelperClient::save_sources_list(&modified)?;
                }
                Ok(())
            }
            "persis-extra" => {
                let deb_line = "deb https://repo.persisos.org/persis trixie main";
                let config_text = std::fs::read_to_string(self.sources_list).unwrap_or_default();
                if !config_text.contains("repo.persisos.org") {
                    let modified = format!("{config_text}\n{deb_line}\n");
                    crate::helper_client::HelperClient::save_sources_list(&modified)?;
                }
                Ok(())
            }
            "liquorix" => {
                let deb_line = "deb http://liquorix.net/debian sid main";
                let config_text = std::fs::read_to_string(self.sources_list).unwrap_or_default();
                if !config_text.contains("liquorix.net") {
                    let modified = format!("{config_text}\n{deb_line}\n");
                    crate::helper_client::HelperClient::save_sources_list(&modified)?;
                }
                Ok(())
            }
            _ => Err(format!("Unknown third-party repo: {repo_name}")),
        }
    }
}

pub fn toggle_repo_text(config_text: &str, repo_name: &str, enable: bool, section_snippet: Option<&str>) -> String {
    let mut new_lines = Vec::new();
    let mut found_section = false;
    let mut in_section = false;
    let section_header = format!("[{repo_name}]");
    let snippet_lines: Vec<&str> = section_snippet.map(|s| s.lines().collect()).unwrap_or_default();

    for line in config_text.lines() {
        let stripped = line.trim();
        let header_check = stripped.trim_start_matches('#').trim();
        if header_check == section_header {
            found_section = true;
            in_section = true;
            if enable {
                new_lines.push(line.trim_start_matches('#').to_string());
                if snippet_lines.len() > 1 {
                    for sl in &snippet_lines[1..] {
                        if sl.starts_with("deb ") || sl.starts_with("URIs:") || sl.starts_with("Components:") {
                            new_lines.push(sl.to_string());
                        }
                    }
                }
            } else {
                if !line.starts_with('#') {
                    new_lines.push(format!("#{line}"));
                } else {
                    new_lines.push(line.to_string());
                }
            }
            continue;
        }

        if in_section {
            let header_uncommented = stripped.trim_start_matches('#').trim();
            if header_uncommented.starts_with('[') && header_uncommented.ends_with(']') {
                in_section = false;
            } else {
                if enable {
                    if snippet_lines.is_empty() {
                        new_lines.push(line.trim_start_matches('#').to_string());
                    }
                } else {
                    if !line.starts_with('#') && !stripped.is_empty() {
                        new_lines.push(format!("#{line}"));
                    } else {
                        new_lines.push(line.to_string());
                    }
                }
                continue;
            }
        }

        new_lines.push(line.to_string());
    }

    if !found_section && enable {
        if let Some(snippet) = section_snippet {
            if !new_lines.is_empty() && !new_lines.last().is_some_and(|l: &String| l.is_empty()) {
                new_lines.push(String::new());
            }
            for sl in snippet.lines() {
                new_lines.push(sl.to_string());
            }
        } else {
            if !new_lines.is_empty() && !new_lines.last().is_some_and(|l: &String| l.is_empty()) {
                new_lines.push(String::new());
            }
            new_lines.push(format!("[{repo_name}]"));
            new_lines.push(format!("deb https://deb.debian.org/debian trixie {repo_name}"));
        }
    }

    new_lines.push(String::new());
    new_lines.join("\n")
}

pub fn remove_repo_text(config_text: &str, repo_name: &str) -> String {
    let mut new_lines = Vec::new();
    let mut in_target_section = false;
    let section_header = format!("[{repo_name}]");
    let mut skip_next_blank = false;

    for line in config_text.lines() {
        let stripped = line.trim();
        let header_check = stripped.trim_start_matches('#').trim();

        if header_check == section_header {
            in_target_section = true;
            skip_next_blank = true;
            continue;
        }

        if in_target_section {
            let header_uncommented = stripped.trim_start_matches('#').trim();
            if header_uncommented.starts_with('[') && header_uncommented.ends_with(']') {
                in_target_section = false;
                skip_next_blank = false;
            } else {
                if stripped.is_empty() && skip_next_blank {
                    continue;
                }
                continue;
            }
        }

        if skip_next_blank {
            if stripped.is_empty() {
                continue;
            }
            skip_next_blank = false;
        }

        new_lines.push(line.to_string());
    }

    new_lines.push(String::new());
    new_lines.join("\n")
}

pub fn get_third_party_section(repo_name: &str) -> Option<String> {
    match repo_name {
        "trixie-backports" => Some("deb https://deb.debian.org/debian trixie-backports main contrib non-free non-free-firmware\n".to_string()),
        "persis-extra" => Some("deb https://repo.persisos.org/persis trixie main\n".to_string()),
        "liquorix" => Some("deb http://liquorix.net/debian sid main\n".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_disable_uncommented_section() {
        let config = "[options]\n\n[contrib]\ndeb http://deb.debian.org/debian trixie contrib\n";
        let result = toggle_repo_text(config, "contrib", false, None);
        assert!(result.contains("#[contrib]"));
        assert!(result.contains("#deb http://deb.debian.org/debian trixie contrib"));
    }

    #[test]
    fn toggle_enable_commented_section() {
        let config = "[options]\n\n#[contrib]\n#deb http://deb.debian.org/debian trixie contrib\n";
        let result = toggle_repo_text(config, "contrib", true, None);
        assert!(result.contains("[contrib]"));
        assert!(result.contains("deb http://deb.debian.org/debian trixie contrib"));
        assert!(!result.contains("#[contrib]"));
    }

    #[test]
    fn toggle_already_enabled_no_change() {
        let config = "[options]\n\n[non-free]\ndeb http://deb.debian.org/debian trixie non-free\n";
        let result = toggle_repo_text(config, "non-free", true, None);
        assert!(result.contains("[non-free]\ndeb http://deb.debian.org/debian trixie non-free"));
    }

    #[test]
    fn toggle_nonexistent_section_adds_it() {
        let config = "[options]\n\n[main]\ndeb http://deb.debian.org/debian trixie main\n";
        let result = toggle_repo_text(config, "contrib", true, None);
        assert!(result.contains("[contrib]"));
        assert!(result.contains("[main]"));
    }

    #[test]
    fn remove_existing_repo() {
        let config = "[options]\n\n[main]\ndeb http://deb.debian.org/debian trixie main\n\n[myrepo]\nServer = https://example.com/\n";
        let result = remove_repo_text(config, "myrepo");
        assert!(!result.contains("[myrepo]"));
        assert!(!result.contains("example.com"));
        assert!(result.contains("[main]"));
    }
}
