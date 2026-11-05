use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mirror {
    pub url: String,
    pub country: String,
    pub country_code: String,
    pub protocol: String,
    pub speed: Option<f64>,
    pub last_sync: Option<String>,
    pub enabled: bool,
    pub ipv4: bool,
    pub ipv6: bool,
    pub completion_pct: Option<f64>,
    pub score: Option<f64>,
    pub duration_avg: Option<f64>,
    pub duration_stddev: Option<f64>,
}

type SpeedTestResults = Arc<Mutex<Vec<(usize, Option<f64>)>>>;

pub fn country_flag(code: &str) -> String {
    if code.len() != 2 { return String::new(); }
    let code = code.to_uppercase();
    let bytes = code.as_bytes();
    let a = bytes[0] as u32;
    let b = bytes[1] as u32;
    if !(65..=90).contains(&a) || !(65..=90).contains(&b) { return String::new(); }
    let ra = char::from_u32(0x1F1E6 + (a - 65)).unwrap_or(' ');
    let rb = char::from_u32(0x1F1E6 + (b - 65)).unwrap_or(' ');
    format!("{}{}", ra, rb)
}

/// Builds the URL used to time a Debian mirror (downloads a lightweight Release index).
pub fn speed_test_url(url: &str) -> String {
    let clean = url.trim_end_matches('/');
    format!("{clean}/dists/stable/Release")
}

/// Builds the URL used for a cheap HEAD or GET availability probe.
pub fn availability_url(url: &str) -> String {
    let clean = url.trim_end_matches('/');
    format!("{clean}/project/trace/master")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub urls: Vec<ApiMirror>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiMirror {
    pub url: Option<String>,
    pub protocol: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub last_sync: Option<String>,
    pub ipv4: Option<bool>,
    pub ipv6: Option<bool>,
    pub completion_pct: Option<f64>,
    pub score: Option<f64>,
    pub duration_avg: Option<f64>,
    pub duration_stddev: Option<f64>,
}

const DEBIAN_MASTERLIST_URL: &str = "https://salsa.debian.org/mirror-team/masterlist/-/raw/master/Mirrors.masterlist.in";
const DEBIAN_MASTERLIST_FALLBACK_URL: &str = "https://salsa.debian.org/webmaster-team/webwml/-/raw/master/english/mirror/Mirrors.masterlist";
const USER_AGENT: &str = "mirrorman/0.6.0 (Persis OS)";

pub const SOURCES_LIST_FILE: &str = "/etc/apt/sources.list";
pub const SOURCES_DEB822_FILE: &str = "/etc/apt/sources.list.d/debian.sources";
pub const SOURCES_BACKUP_FILE: &str = "/etc/apt/sources.list.backup";
pub const SOURCES_DEB822_BACKUP: &str = "/etc/apt/sources.list.d/debian.sources.backup";

// Legacy fallback constants for compatibility
pub const MIRRORLIST_FILE: &str = SOURCES_LIST_FILE;
pub const MIRRORLIST_BACKUP: &str = SOURCES_BACKUP_FILE;

pub const IRANIAN_MIRRORS: &[&str] = &[
    "https://mirror.iranserver.com/debian",
    "https://mirror.mobinhost.com/debian",
    "https://repo.iut.ac.ir/debian",
    "https://mirror.rasanegar.com/debian",
    "https://mirror.aminidc.com/debian",
];

/// Curated fallback mirrors for Debian GNU/Linux across major geographic regions.
const CURATED_DEBIAN_MIRRORS: &[(&str, &str, &str, &str, bool, bool)] = &[
    ("https://deb.debian.org/debian", "Worldwide", "WW", "https", true, true),
    ("http://deb.debian.org/debian", "Worldwide", "WW", "http", true, true),
    ("https://mirror.iranserver.com/debian", "Iran", "IR", "https", true, false),
    ("https://mirror.mobinhost.com/debian", "Iran", "IR", "https", true, false),
    ("https://repo.iut.ac.ir/debian", "Iran", "IR", "https", true, false),
    ("https://mirror.rasanegar.com/debian", "Iran", "IR", "https", true, false),
    ("https://mirror.aminidc.com/debian", "Iran", "IR", "https", true, false),
    ("http://ftp.de.debian.org/debian", "Germany", "DE", "http", true, true),
    ("https://debian.charite.de/debian", "Germany", "DE", "https", true, true),
    ("https://mirror.leaseweb.com/debian", "Germany", "DE", "https", true, true),
    ("http://ftp.fr.debian.org/debian", "France", "FR", "http", true, true),
    ("https://debian.polytechnique.fr/debian", "France", "FR", "https", true, true),
    ("http://ftp.nl.debian.org/debian", "Netherlands", "NL", "http", true, true),
    ("https://debian.snt.utwente.nl/debian", "Netherlands", "NL", "https", true, true),
    ("http://ftp.uk.debian.org/debian", "United Kingdom", "GB", "http", true, true),
    ("https://mirrors.ukfast.co.uk/sites/debian", "United Kingdom", "GB", "https", true, false),
    ("http://ftp.us.debian.org/debian", "United States", "US", "http", true, true),
    ("https://mirrors.kernel.org/debian", "United States", "US", "https", true, true),
    ("https://mirror.math.princeton.edu/pub/debian", "United States", "US", "https", true, true),
    ("https://mirror.csclub.uwaterloo.ca/debian", "Canada", "CA", "https", true, true),
    ("http://ftp.jp.debian.org/debian", "Japan", "JP", "http", true, true),
    ("http://ftp.kr.debian.org/debian", "South Korea", "KR", "http", true, true),
    ("https://mirrors.ustc.edu.cn/debian", "China", "CN", "https", true, true),
    ("https://mirrors.tuna.tsinghua.edu.cn/debian", "China", "CN", "https", true, true),
    ("http://ftp.br.debian.org/debian", "Brazil", "BR", "http", true, true),
    ("http://ftp.au.debian.org/debian", "Australia", "AU", "http", true, true),
    ("https://mirror.aarnet.edu.au/debian", "Australia", "AU", "https", true, true),
    ("http://ftp.tr.debian.org/debian", "Turkey", "TR", "http", true, false),
    ("https://mirror.netonomy.ch/debian", "Switzerland", "CH", "https", true, true),
    ("http://ftp.se.debian.org/debian", "Sweden", "SE", "http", true, true),
    ("http://ftp.fi.debian.org/debian", "Finland", "FI", "http", true, true),
    ("http://ftp.it.debian.org/debian", "Italy", "IT", "http", true, true),
    ("http://ftp.es.debian.org/debian", "Spain", "ES", "http", true, true),
    ("http://ftp.at.debian.org/debian", "Austria", "AT", "http", true, true),
    ("http://ftp.pl.debian.org/debian", "Poland", "PL", "http", true, true),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusCache {
    pub cached_at: DateTime<Utc>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub raw_json: String,
}

pub fn status_cache_path() -> std::path::PathBuf {
    if let Ok(p) = std::env::var("MIRRORMAN_STATUS_CACHE") {
        return std::path::PathBuf::from(p);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    let cache_base = std::env::var("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from(format!("{home}/.cache")));
    let mut dir = cache_base;
    dir.push("mirrorman");
    let _ = std::fs::create_dir_all(&dir);
    dir.push("debian_status.json");
    dir
}

pub fn load_status_cache() -> Option<StatusCache> {
    let path = status_cache_path();
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_status_cache(cache: &StatusCache) {
    let path = status_cache_path();
    if let Ok(json) = serde_json::to_string(cache) {
        let _ = std::fs::write(path, json);
    }
}

/// Parses Debian's Mirrors.masterlist format into a list of ApiMirror objects.
pub fn parse_debian_masterlist(text: &str) -> Vec<ApiMirror> {
    let mut results = Vec::new();

    for block in text.split("\n\n") {
        let mut site = "";
        let mut country_code = "";
        let mut country = "";
        let mut http_path = "";
        let mut ipv6 = false;

        for line in block.lines() {
            let line = line.trim();
            if let Some((k, v)) = line.split_once(':') {
                let key = k.trim();
                let val = v.trim();
                match key {
                    "Site" => site = val,
                    "Country" => {
                        let parts: Vec<&str> = val.split_whitespace().collect();
                        if !parts.is_empty() {
                            country_code = parts[0];
                            if parts.len() > 1 {
                                country = &val[parts[0].len()..].trim();
                            } else {
                                country = parts[0];
                            }
                        }
                    }
                    "Archive-http" => http_path = val,
                    "IPv6" => ipv6 = val.eq_ignore_ascii_case("yes"),
                    _ => {}
                }
            }
        }

        if !site.is_empty() && !http_path.is_empty() {
            let clean_path = http_path.trim_matches('/');
            let base_path = if clean_path.is_empty() { String::new() } else { format!("/{clean_path}") };
            let url = format!("https://{site}{base_path}");

            results.push(ApiMirror {
                url: Some(url),
                protocol: Some("https".to_string()),
                country: Some(country.to_string()),
                country_code: Some(country_code.to_string()),
                last_sync: None,
                ipv4: Some(true),
                ipv6: Some(ipv6),
                completion_pct: Some(1.0),
                score: None,
                duration_avg: None,
                duration_stddev: None,
            });

            // Also add HTTP variant
            let http_url = format!("http://{site}{base_path}");
            results.push(ApiMirror {
                url: Some(http_url),
                protocol: Some("http".to_string()),
                country: Some(country.to_string()),
                country_code: Some(country_code.to_string()),
                last_sync: None,
                ipv4: Some(true),
                ipv6: Some(ipv6),
                completion_pct: Some(1.0),
                score: None,
                duration_avg: None,
                duration_stddev: None,
            });
        }
    }

    results
}

pub fn get_curated_mirrors() -> Vec<ApiMirror> {
    CURATED_DEBIAN_MIRRORS
        .iter()
        .map(|&(url, country, code, proto, v4, v6)| ApiMirror {
            url: Some(url.to_string()),
            protocol: Some(proto.to_string()),
            country: Some(country.to_string()),
            country_code: Some(code.to_string()),
            last_sync: None,
            ipv4: Some(v4),
            ipv6: Some(v6),
            completion_pct: Some(1.0),
            score: None,
            duration_avg: None,
            duration_stddev: None,
        })
        .collect()
}

pub fn create_http_client(timeout: std::time::Duration) -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new())
}

pub fn create_direct_http_client(timeout: std::time::Duration) -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(timeout)
        .no_proxy()
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new())
}

pub fn fetch_raw_api_status(force_network: bool) -> Result<String, String> {
    let cached = load_status_cache();

    if !force_network {
        if let Some(ref c) = cached {
            let age = Utc::now() - c.cached_at;
            if age < Duration::hours(6) && !c.raw_json.is_empty() {
                return Ok(c.raw_json.clone());
            }
        }
    }

    let client = create_http_client(std::time::Duration::from_secs(10));
    let direct_client = create_direct_http_client(std::time::Duration::from_secs(10));

    // Try fetching with standard client, then direct client, then fallback URL
    let urls = [DEBIAN_MASTERLIST_URL, DEBIAN_MASTERLIST_FALLBACK_URL];
    let mut fetched_body = None;
    let mut etag = None;
    let mut last_modified = None;

    for target_url in urls {
        for http_client in [&client, &direct_client] {
            let mut req = http_client.get(target_url);
            if let Some(ref c) = cached {
                if let Some(ref et) = c.etag {
                    req = req.header("If-None-Match", et);
                }
                if let Some(ref lm) = c.last_modified {
                    req = req.header("If-Modified-Since", lm);
                }
            }

            if let Ok(resp) = req.send() {
                if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
                    if let Some(mut c) = cached.clone() {
                        c.cached_at = Utc::now();
                        save_status_cache(&c);
                        return Ok(c.raw_json);
                    }
                }
                if resp.status().is_success() {
                    etag = resp
                        .headers()
                        .get("ETag")
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.to_string());
                    last_modified = resp
                        .headers()
                        .get("Last-Modified")
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.to_string());
                    if let Ok(text) = resp.text() {
                        if !text.trim().is_empty() {
                            fetched_body = Some(text);
                            break;
                        }
                    }
                }
            }
        }
        if fetched_body.is_some() {
            break;
        }
    }

    if let Some(body) = fetched_body {
        let parsed = parse_debian_masterlist(&body);
        let json_data = if parsed.is_empty() {
            let fallback = ApiResponse { urls: get_curated_mirrors() };
            serde_json::to_string(&fallback).unwrap_or_default()
        } else {
            let response_obj = ApiResponse { urls: parsed };
            serde_json::to_string(&response_obj).unwrap_or_default()
        };

        let new_cache = StatusCache {
            cached_at: Utc::now(),
            etag,
            last_modified,
            raw_json: json_data.clone(),
        };
        save_status_cache(&new_cache);
        Ok(json_data)
    } else if let Some(c) = cached {
        Ok(c.raw_json)
    } else {
        let fallback = ApiResponse { urls: get_curated_mirrors() };
        serde_json::to_string(&fallback).map_err(|e| e.to_string())
    }
}

pub fn country_selected(countries: &[String], mirror_country: &str) -> bool {
    countries.is_empty() || countries.iter().any(|c| c == mirror_country)
}

pub struct MirrorManager {
    pub mirrors: Vec<Mirror>,
    pub countries: Vec<String>,
}

impl Default for MirrorManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MirrorManager {
    pub fn new() -> Self {
        Self {
            mirrors: Vec::new(),
            countries: Vec::new(),
        }
    }

    pub fn fetch_mirrors(
        &mut self,
        countries: &[String],
        protocols: &[String],
        ip_versions: &[String],
        _use_status: bool,
    ) -> Result<(), String> {
        let body = fetch_raw_api_status(false).unwrap_or_else(|_| {
            let fallback = ApiResponse { urls: get_curated_mirrors() };
            serde_json::to_string(&fallback).unwrap_or_default()
        });

        let mut api: ApiResponse = serde_json::from_str(&body).unwrap_or_else(|_| {
            ApiResponse { urls: get_curated_mirrors() }
        });

        if api.urls.is_empty() {
            api.urls = get_curated_mirrors();
        }

        let mut countries_set = std::collections::BTreeSet::new();
        let mut mirrors = Vec::new();

        let want_ipv4 = ip_versions.contains(&"4".to_string());
        let want_ipv6 = ip_versions.contains(&"6".to_string());
        if !want_ipv4 && !want_ipv6 {
            return Err("Select at least one IP version".to_string());
        }

        for m in api.urls {
            let mirror_country = m.country.unwrap_or_else(|| "Unknown".to_string());
            if !mirror_country.is_empty() && mirror_country != "Unknown" {
                countries_set.insert(mirror_country.clone());
            }

            if !country_selected(countries, &mirror_country) {
                continue;
            }

            let protocol = m.protocol.unwrap_or_else(|| "https".to_string());
            if !protocols.is_empty() && !protocols.contains(&protocol.to_lowercase()) {
                continue;
            }

            let url = match m.url {
                Some(u) if !u.is_empty() => u,
                _ => continue,
            };

            let mirror_ipv4 = m.ipv4.unwrap_or(true);
            let mirror_ipv6 = m.ipv6.unwrap_or(false);
            let has_wanted_ip = (want_ipv4 && mirror_ipv4) || (want_ipv6 && mirror_ipv6);
            if !has_wanted_ip {
                continue;
            }

            let country_code = m.country_code.unwrap_or_default();

            mirrors.push(Mirror {
                url,
                country: mirror_country,
                country_code,
                protocol,
                speed: None,
                last_sync: m.last_sync,
                enabled: true,
                ipv4: mirror_ipv4,
                ipv6: mirror_ipv6,
                completion_pct: m.completion_pct.or(Some(1.0)),
                score: m.score,
                duration_avg: m.duration_avg,
                duration_stddev: m.duration_stddev,
            });
        }

        countries_set.insert("Worldwide".to_string());
        self.countries = countries_set.into_iter().collect();
        self.countries.sort();
        self.mirrors = mirrors;

        Ok(())
    }

    pub fn fetch_countries_only(&self) -> Result<Vec<String>, String> {
        let body = fetch_raw_api_status(false).unwrap_or_else(|_| {
            let fallback = ApiResponse { urls: get_curated_mirrors() };
            serde_json::to_string(&fallback).unwrap_or_default()
        });

        let api: ApiResponse = serde_json::from_str(&body).unwrap_or_else(|_| {
            ApiResponse { urls: get_curated_mirrors() }
        });

        let mut countries: std::collections::BTreeSet<String> = api
            .urls
            .iter()
            .filter_map(|m| {
                let c = m.country.as_deref().unwrap_or("");
                if c.is_empty() || c == "Unknown" {
                    None
                } else {
                    Some(c.to_string())
                }
            })
            .collect();

        countries.insert("Worldwide".to_string());
        let mut list: Vec<_> = countries.into_iter().collect();
        list.sort();
        Ok(list)
    }

    pub fn test_all_speeds_concurrent(mirrors: &mut [Mirror], max_workers: usize) {
        if mirrors.is_empty() {
            return;
        }

        let num_workers = max_workers.clamp(1, mirrors.len().max(1));
        let results: SpeedTestResults = Arc::new(Mutex::new(Vec::new()));

        let mirrors_arc = Arc::new(mirrors.to_vec());
        let total_mirrors = mirrors.len();

        let chunk_size = (total_mirrors + num_workers - 1) / num_workers;
        let mut handles = Vec::new();

        for worker_id in 0..num_workers {
            let results_clone = Arc::clone(&results);
            let mirrors_ref = Arc::clone(&mirrors_arc);

            let start_idx = worker_id * chunk_size;
            let end_idx = (start_idx + chunk_size).min(total_mirrors);

            if start_idx >= total_mirrors {
                break;
            }

            let handle = std::thread::spawn(move || {
                let client = create_http_client(std::time::Duration::from_secs(4));
                let direct_client = create_direct_http_client(std::time::Duration::from_secs(4));

                for idx in start_idx..end_idx {
                    let mirror = &mirrors_ref[idx];
                    let speed = Self::test_mirror_speed_with_client(&client, &direct_client, &mirror.url);
                    let mut res = results_clone.lock().unwrap_or_else(|e| e.into_inner());
                    res.push((idx, speed));
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }

        let results_guard = results.lock().unwrap_or_else(|e| e.into_inner());
        for &(idx, speed) in results_guard.iter() {
            if idx < mirrors.len() {
                mirrors[idx].speed = speed;
            }
        }
    }

    fn test_mirror_speed_with_client(
        client: &reqwest::blocking::Client,
        direct_client: &reqwest::blocking::Client,
        url: &str,
    ) -> Option<f64> {
        let test_url = speed_test_url(url);

        let start = Instant::now();
        if let Ok(resp) = client.get(&test_url).send() {
            if resp.status().is_success() || resp.status() == reqwest::StatusCode::PARTIAL_CONTENT {
                let duration = start.elapsed();
                return Some(duration.as_secs_f64() * 1000.0);
            }
        }

        // Direct fallback when environment proxy is invalid or fails
        let start = Instant::now();
        if let Ok(resp) = direct_client.get(&test_url).send() {
            if resp.status().is_success() || resp.status() == reqwest::StatusCode::PARTIAL_CONTENT {
                let duration = start.elapsed();
                return Some(duration.as_secs_f64() * 1000.0);
            }
        }

        None
    }

    pub fn test_single_mirror_speed(url: &str) -> Option<f64> {
        let client = create_http_client(std::time::Duration::from_secs(4));
        let direct_client = create_direct_http_client(std::time::Duration::from_secs(4));
        Self::test_mirror_speed_with_client(&client, &direct_client, url)
    }

    pub fn check_availability_concurrent(mirrors: &mut [Mirror], max_workers: usize) {
        if mirrors.is_empty() {
            return;
        }

        let num_workers = max_workers.clamp(1, mirrors.len().max(1));
        let results: Arc<Mutex<Vec<(usize, bool)>>> = Arc::new(Mutex::new(Vec::new()));

        let mirrors_arc = Arc::new(mirrors.to_vec());
        let total_mirrors = mirrors.len();

        let chunk_size = (total_mirrors + num_workers - 1) / num_workers;
        let mut handles = Vec::new();

        for worker_id in 0..num_workers {
            let results_clone = Arc::clone(&results);
            let mirrors_ref = Arc::clone(&mirrors_arc);

            let start_idx = worker_id * chunk_size;
            let end_idx = (start_idx + chunk_size).min(total_mirrors);

            if start_idx >= total_mirrors {
                break;
            }

            let handle = std::thread::spawn(move || {
                let client = create_http_client(std::time::Duration::from_secs(4));
                let direct_client = create_direct_http_client(std::time::Duration::from_secs(4));

                for idx in start_idx..end_idx {
                    let mirror = &mirrors_ref[idx];
                    let avail = Self::check_single_availability(&client, &direct_client, &mirror.url);
                    let mut res = results_clone.lock().unwrap_or_else(|e| e.into_inner());
                    res.push((idx, avail));
                }
            });

            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.join();
        }

        let results_guard = results.lock().unwrap_or_else(|e| e.into_inner());
        for &(idx, avail) in results_guard.iter() {
            if idx < mirrors.len() {
                mirrors[idx].enabled = avail;
            }
        }
    }

    fn probe_url(client: &reqwest::blocking::Client, url: &str) -> bool {
        let clean = url.trim_end_matches('/');

        // 1. Primary probe: Debian trace master file via HEAD
        let trace = availability_url(url);
        if let Ok(resp) = client.head(&trace).send() {
            let status = resp.status();
            if status.is_success() || status.is_redirection() {
                return true;
            }
        }

        // 2. Secondary probe: Stable Release index via HEAD
        let release = speed_test_url(url);
        if let Ok(resp) = client.head(&release).send() {
            let status = resp.status();
            if status.is_success() || status.is_redirection() {
                return true;
            }
        }

        // 3. Tertiary probe: Base mirror root via HEAD
        let root = format!("{clean}/");
        if let Ok(resp) = client.head(&root).send() {
            let status = resp.status();
            if status.is_success() || status.is_redirection() {
                return true;
            }
        }

        // 4. Fallback GET with small range header if server disallows or fails on HEAD
        if let Ok(resp) = client
            .get(&release)
            .header("Range", "bytes=0-128")
            .send()
        {
            let status = resp.status();
            if status.is_success()
                || status.is_redirection()
                || status == reqwest::StatusCode::PARTIAL_CONTENT
            {
                return true;
            }
        }

        false
    }

    pub fn check_single_availability(
        client: &reqwest::blocking::Client,
        direct_client: &reqwest::blocking::Client,
        url: &str,
    ) -> bool {
        if Self::probe_url(client, url) {
            return true;
        }
        Self::probe_url(direct_client, url)
    }

    pub fn check_mirror_availability(mirrors: &mut [Mirror], max_workers: usize) {
        Self::check_availability_concurrent(mirrors, max_workers);
    }

    pub fn add_iran_mirrors(&mut self) {
        for &url in IRANIAN_MIRRORS {
            if !self.mirrors.iter().any(|m| m.url == url) {
                self.mirrors.insert(
                    0,
                    Mirror {
                        url: url.to_string(),
                        country: "Iran".to_string(),
                        country_code: "IR".to_string(),
                        protocol: if url.starts_with("https") {
                            "https".to_string()
                        } else {
                            "http".to_string()
                        },
                        speed: None,
                        last_sync: None,
                        enabled: true,
                        ipv4: true,
                        ipv6: false,
                        completion_pct: Some(1.0),
                        score: Some(0.0),
                        duration_avg: None,
                        duration_stddev: None,
                    },
                );
            }
        }
    }

    pub fn sort_by_reliability(&mut self) {
        self.mirrors.sort_by(|a, b| match (b.completion_pct, a.completion_pct) {
            (Some(p1), Some(p2)) => p1.partial_cmp(&p2).unwrap_or(std::cmp::Ordering::Equal),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
    }

    pub fn sort_by_speed(&mut self) {
        self.mirrors.sort_by(|a, b| match (a.speed, b.speed) {
            (Some(s1), Some(s2)) => s1.partial_cmp(&s2).unwrap_or(std::cmp::Ordering::Equal),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
    }

    pub fn sort_by_score(&mut self) {
        self.mirrors.sort_by(|a, b| match (a.score, b.score) {
            (Some(s1), Some(s2)) => s1.partial_cmp(&s2).unwrap_or(std::cmp::Ordering::Equal),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
    }

    pub fn sort_by_country(&mut self) {
        self.mirrors.sort_by(|a, b| a.country.cmp(&b.country));
    }

    pub fn sort_by_age(&mut self) {
        self.mirrors.sort_by(|a, b| match (&a.last_sync, &b.last_sync) {
            (Some(a), Some(b)) => b.cmp(a),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
    }

    pub fn auto_optimize(&mut self) -> Vec<Mirror> {
        self.auto_optimize_count(5)
    }

    pub fn auto_optimize_with_count(&mut self, max_mirrors: usize) -> Vec<Mirror> {
        self.auto_optimize_count(max_mirrors)
    }

    pub fn auto_optimize_count(&mut self, max_mirrors: usize) -> Vec<Mirror> {
        self.sort_by_speed();

        let mut country_counts = std::collections::HashMap::new();
        let mut selected_indices = std::collections::HashSet::new();

        for (idx, m) in self.mirrors.iter().enumerate() {
            if selected_indices.len() >= max_mirrors {
                break;
            }
            if m.speed.is_none() {
                continue;
            }

            let count = country_counts.entry(m.country.clone()).or_insert(0);
            if *count < 2 {
                selected_indices.insert(idx);
                *count += 1;
            }
        }

        if selected_indices.len() < max_mirrors {
            for (idx, m) in self.mirrors.iter().enumerate() {
                if selected_indices.len() >= max_mirrors {
                    break;
                }
                if m.speed.is_some() && !selected_indices.contains(&idx) {
                    selected_indices.insert(idx);
                }
            }
        }

        let mut selected_mirrors = Vec::new();
        for (idx, m) in self.mirrors.iter_mut().enumerate() {
            if selected_indices.contains(&idx) {
                m.enabled = true;
                selected_mirrors.push(m.clone());
            } else {
                m.enabled = false;
            }
        }

        selected_mirrors
    }

    /// Detects whether the system is primarily configured for DEB822 sources format.
    pub fn is_deb822_primary() -> bool {
        std::path::Path::new(SOURCES_DEB822_FILE).exists()
    }

    pub fn read_current_mirrorlist() -> String {
        if Self::is_deb822_primary() {
            std::fs::read_to_string(SOURCES_DEB822_FILE).unwrap_or_default()
        } else {
            std::fs::read_to_string(SOURCES_LIST_FILE).unwrap_or_default()
        }
    }

    pub fn save_mirrorlist(&self) -> Result<(), String> {
        let enabled_count = self.mirrors.iter().filter(|m| m.enabled).count();
        if enabled_count == 0 {
            return Err(
                "Refusing to save empty sources: no mirrors are enabled. Run 'refresh' and 'best-setup' first, or use --force."
                    .to_string(),
            );
        }

        // Save traditional /etc/apt/sources.list
        let content_traditional = self.generate_sources_list_content();
        crate::helper_client::HelperClient::save_sources_list(&content_traditional)?;

        // If DEB822 file exists, also synchronize DEB822 format
        if Self::is_deb822_primary() {
            let content_deb822 = self.generate_deb822_content();
            let _ = crate::helper_client::HelperClient::save_deb822_sources(&content_deb822);
        }

        Ok(())
    }

    /// Generates standard Debian APT sources.list configuration.
    pub fn generate_sources_list_content(&self) -> String {
        let mut s = String::new();
        s.push_str("##\n## Persis OS / Debian repository sources.list\n");
        s.push_str("## Generated by mirrorman\n##\n\n");

        let enabled_mirrors: Vec<_> = self.mirrors.iter().filter(|m| m.enabled).collect();
        s.push_str(&format!("## {} enabled mirror(s)\n\n", enabled_mirrors.len()));

        let default_url = "https://deb.debian.org/debian";
        let primary_url = enabled_mirrors
            .first()
            .map(|m| m.url.trim_end_matches('/'))
            .unwrap_or(default_url);

        s.push_str(&format!("deb {primary_url} trixie main contrib non-free non-free-firmware\n"));
        s.push_str(&format!("deb-src {primary_url} trixie main contrib non-free non-free-firmware\n\n"));

        s.push_str(&format!("deb {primary_url} trixie-updates main contrib non-free non-free-firmware\n"));
        s.push_str(&format!("deb-src {primary_url} trixie-updates main contrib non-free non-free-firmware\n\n"));

        s.push_str("deb http://security.debian.org/debian-security trixie-security main contrib non-free non-free-firmware\n");
        s.push_str("deb-src http://security.debian.org/debian-security trixie-security main contrib non-free non-free-firmware\n\n");

        if enabled_mirrors.len() > 1 {
            s.push_str("## Alternative enabled mirrors:\n");
            for m in &enabled_mirrors[1..] {
                let u = m.url.trim_end_matches('/');
                s.push_str(&format!("# deb {u} trixie main contrib non-free non-free-firmware\n"));
            }
            s.push('\n');
        }

        s
    }

    /// Generates modern DEB822 (.sources) format.
    pub fn generate_deb822_content(&self) -> String {
        let mut s = String::new();
        s.push_str("##\n## Persis OS / Debian DEB822 repository configuration\n");
        s.push_str("## Generated by mirrorman\n##\n\n");

        let enabled_mirrors: Vec<_> = self.mirrors.iter().filter(|m| m.enabled).collect();
        let default_url = "https://deb.debian.org/debian";
        let primary_url = enabled_mirrors
            .first()
            .map(|m| m.url.trim_end_matches('/'))
            .unwrap_or(default_url);

        s.push_str("Types: deb deb-src\n");
        s.push_str(&format!("URIs: {primary_url}\n"));
        s.push_str("Suites: trixie trixie-updates\n");
        s.push_str("Components: main contrib non-free non-free-firmware\n");
        s.push_str("Signed-By: /usr/share/keyrings/debian-archive-keyring.gpg\n\n");

        s.push_str("Types: deb deb-src\n");
        s.push_str("URIs: http://security.debian.org/debian-security\n");
        s.push_str("Suites: trixie-security\n");
        s.push_str("Components: main contrib non-free non-free-firmware\n");
        s.push_str("Signed-By: /usr/share/keyrings/debian-archive-keyring.gpg\n");

        s
    }

    /// Generates proposed repository sources content (DEB822 or traditional sources.list).
    pub fn generate_mirrorlist_content(&self) -> String {
        if Self::is_deb822_primary() {
            self.generate_deb822_content()
        } else {
            self.generate_sources_list_content()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mirror_sorting_by_score() {
        let mut mgr = MirrorManager::new();
        mgr.mirrors = vec![
            Mirror {
                url: "http://mirror1.org/debian".to_string(),
                country: "Germany".to_string(),
                country_code: "DE".to_string(),
                protocol: "https".to_string(),
                speed: Some(150.0),
                last_sync: None,
                enabled: true,
                ipv4: true,
                ipv6: false,
                completion_pct: Some(1.0),
                score: Some(2.5),
                duration_avg: None,
                duration_stddev: None,
            },
            Mirror {
                url: "http://mirror2.org/debian".to_string(),
                country: "France".to_string(),
                country_code: "FR".to_string(),
                protocol: "https".to_string(),
                speed: Some(100.0),
                last_sync: None,
                enabled: true,
                ipv4: true,
                ipv6: false,
                completion_pct: Some(1.0),
                score: Some(1.1),
                duration_avg: None,
                duration_stddev: None,
            },
        ];

        mgr.sort_by_score();
        assert_eq!(mgr.mirrors[0].score, Some(1.1));
        assert_eq!(mgr.mirrors[1].score, Some(2.5));
    }

    #[test]
    fn test_auto_optimize_country_diversity() {
        let mut mgr = MirrorManager::new();
        mgr.mirrors = vec![
            Mirror {
                url: "http://m1.de/debian".to_string(),
                country: "Germany".to_string(),
                country_code: "DE".to_string(),
                protocol: "https".to_string(),
                speed: Some(50.0),
                last_sync: None,
                enabled: false,
                ipv4: true,
                ipv6: false,
                completion_pct: Some(1.0),
                score: Some(1.0),
                duration_avg: None,
                duration_stddev: None,
            },
            Mirror {
                url: "http://m2.de/debian".to_string(),
                country: "Germany".to_string(),
                country_code: "DE".to_string(),
                protocol: "https".to_string(),
                speed: Some(60.0),
                last_sync: None,
                enabled: false,
                ipv4: true,
                ipv6: false,
                completion_pct: Some(1.0),
                score: Some(1.2),
                duration_avg: None,
                duration_stddev: None,
            },
            Mirror {
                url: "http://m1.fr/debian".to_string(),
                country: "France".to_string(),
                country_code: "FR".to_string(),
                protocol: "https".to_string(),
                speed: Some(55.0),
                last_sync: None,
                enabled: false,
                ipv4: true,
                ipv6: false,
                completion_pct: Some(1.0),
                score: Some(1.1),
                duration_avg: None,
                duration_stddev: None,
            },
        ];

        let selected = mgr.auto_optimize_count(2);
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].country, "Germany");
        assert_eq!(selected[1].country, "France");
    }

    #[test]
    fn test_generate_sources_content() {
        let mut mgr = MirrorManager::new();
        mgr.mirrors = vec![Mirror {
            url: "https://mirror.iranserver.com/debian".to_string(),
            country: "Iran".to_string(),
            country_code: "IR".to_string(),
            protocol: "https".to_string(),
            speed: Some(25.0),
            last_sync: None,
            enabled: true,
            ipv4: true,
            ipv6: false,
            completion_pct: None,
            score: None,
            duration_avg: None,
            duration_stddev: None,
        }];

        let content = mgr.generate_sources_list_content();
        assert!(content.contains("Persis OS / Debian"));
        assert!(content.contains("deb https://mirror.iranserver.com/debian trixie main"));
        assert!(content.contains("deb-src https://mirror.iranserver.com/debian trixie main"));
        assert!(content.contains("security.debian.org"));

        let deb822 = mgr.generate_deb822_content();
        assert!(deb822.contains("URIs: https://mirror.iranserver.com/debian"));
        assert!(deb822.contains("Suites: trixie trixie-updates"));
        assert!(deb822.contains("Components: main contrib non-free non-free-firmware"));
    }

    #[test]
    fn test_speed_test_url_has_slash() {
        let url = speed_test_url("https://deb.debian.org/debian/");
        assert_eq!(url, "https://deb.debian.org/debian/dists/stable/Release");
    }

    #[test]
    fn test_availability_url_probes_trace() {
        let url = availability_url("https://deb.debian.org/debian/");
        assert_eq!(url, "https://deb.debian.org/debian/project/trace/master");
    }

    #[test]
    fn test_country_selected_empty_means_all() {
        assert!(country_selected(&[], "Germany"));
        assert!(country_selected(&[], "Iran"));
    }

    #[test]
    fn test_country_selected_multi() {
        let filter = vec!["Germany".to_string(), "Iran".to_string()];
        assert!(country_selected(&filter, "Germany"));
        assert!(country_selected(&filter, "Iran"));
        assert!(!country_selected(&filter, "France"));
    }

    #[test]
    fn test_parse_debian_masterlist() {
        let sample = "\
Site: debian.mirror.at
Country: AT Austria
Archive-http: /debian/
IPv6: yes

Site: ftp.de.debian.org
Country: DE Germany
Archive-http: /debian/
IPv6: no
";
        let parsed = parse_debian_masterlist(sample);
        assert_eq!(parsed.len(), 4); // 2 HTTPS + 2 HTTP
        assert!(parsed.iter().any(|m| m.url.as_deref() == Some("https://debian.mirror.at/debian")));
        assert!(parsed.iter().any(|m| m.country_code.as_deref() == Some("AT")));
        assert!(parsed.iter().any(|m| m.country.as_deref() == Some("Austria")));
    }
}
