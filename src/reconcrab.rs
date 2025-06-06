use clap::{Arg, Command};
use reqwest::{Client, StatusCode};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use sysinfo::System;
use tokio::io::{AsyncBufReadExt, BufReader as AsyncBufReader};
use tokio::fs::File as AsyncFile;
use tokio::sync::{Semaphore, Notify};
use tokio::time::timeout;
use url::Url;
use rand::Rng;

// CONSTANTS

const VALID_STATUS_CODES: &[u16] = &[
    200, 201, 202, 203, 204, 205, 206, 207, 208, 226,
    300, 301, 302, 303, 304, 305, 307, 308,
    400, 401, 402, 403, 405, 406, 407, 408, 409, 410, 411, 412, 413, 414, 415, 416, 417, 418, 421, 422, 423, 424, 425, 426, 428, 429, 431, 451,
    500, 501, 502, 503, 504, 505, 506, 507, 508, 510, 511
];

const USER_AGENTS: &[&str] = &[
"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/53.0.2785.143 Safari/537.36",
                        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/54.0.2840.71 Safari/537.36",
                        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_6) AppleWebKit/602.1.50 (KHTML, like Gecko) Version/10.0 Safari/602.1.50",
                        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.11; rv:49.0) Gecko/20100101 Firefox/49.0",
                        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/53.0.2785.143 Safari/537.36",
                        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_0) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/54.0.2840.71 Safari/537.36",
                        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_1) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/54.0.2840.71 Safari/537.36",
                        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12_1) AppleWebKit/602.2.14 (KHTML, like Gecko) Version/10.0.1 Safari/602.2.14",
                        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_12) AppleWebKit/602.1.50 (KHTML, like Gecko) Version/10.0 Safari/602.1.50",
                        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/51.0.2704.79 Safari/537.36 Edge/14.14393",
                        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/53.0.2785.143 Safari/537.36",
                        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/54.0.2840.71 Safari/537.36",
                        "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/53.0.2785.143 Safari/537.36",
                        "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/54.0.2840.71 Safari/537.36",
                        "Mozilla/5.0 (Windows NT 10.0; WOW64; rv:49.0) Gecko/20100101 Firefox/49.0",
                        "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/53.0.2785.143 Safari/537.36",
                        "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/54.0.2840.71 Safari/537.36",
                        "Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/53.0.2785.143 Safari/537.36",
                        "Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/54.0.2840.71 Safari/537.36",
                        "Mozilla/5.0 (Windows NT 6.1; WOW64; rv:49.0) Gecko/20100101 Firefox/49.0",
                        "Mozilla/5.0 (Windows NT 6.1; WOW64; Trident/7.0; rv:11.0) like Gecko",
                        "Mozilla/5.0 (Windows NT 6.3; rv:36.0) Gecko/20100101 Firefox/36.0",
                        "Mozilla/5.0 (Windows NT 6.3; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/53.0.2785.143 Safari/537.36",
                        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/53.0.2785.143 Safari/537.36",
                        "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:49.0) Gecko/20100101 Firefox/49.0"
];

// CONFIG STRUCTURE

#[derive(Clone)]
struct Config {
    target_url: String,
    wordlist_file: String,
    concurrent_requests: usize,
    headers: HashMap<String, String>,
    cookies: HashMap<String, String>,
    timeout: Duration,
    max_retries: u8,
    max_cpu: f32,
}

impl Config {
    fn new(
        target_url: String,
        wordlist_file: String,
        concurrent_requests: Option<usize>,
        headers: Option<Vec<String>>,
        cookies: Option<Vec<String>>,
        max_cpu: Option<f32>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let concurrent_requests = concurrent_requests.unwrap_or(500000);
        let headers = Self::parse_key_value_pairs(headers.unwrap_or_default())?;
        let cookies = Self::parse_key_value_pairs(cookies.unwrap_or_default())?;

        let target_url = if target_url.starts_with("http://") || target_url.starts_with("https://") {
            target_url
        } else {
            format!("http://{}", target_url)
        };
        let target_url = target_url.trim_end_matches('/').to_string();

        let max_cpu = max_cpu.unwrap_or(50.0);
        if !(1.0..=100.0).contains(&max_cpu) {
            return Err(format!("Max CPU must be between 1.0 and 100.0, got {}", max_cpu).into());
        }

        Ok(Config {
            target_url,
            wordlist_file,
            concurrent_requests,
            headers,
            cookies,
            timeout: Duration::from_secs(15),
            max_retries: 2,
            max_cpu,
        })
    }

    fn parse_key_value_pairs(pairs: Vec<String>) -> Result<HashMap<String, String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut map = HashMap::new();
        for pair in pairs {
            let parts: Vec<&str> = pair.splitn(2, ':').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid key-value pair format: {}", pair).into());
            }
            map.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
        }
        Ok(map)
    }
}

#[derive(Copy, Clone)]
enum FuzzMode {
    Subdomain,
    Directory,
}

// HTTP CLIENT BUILDER

fn build_http_client(config: &Config) -> Result<Client, Box<dyn std::error::Error + Send + Sync>> {
    let mut client_builder = Client::builder()
        .timeout(config.timeout)
        .pool_max_idle_per_host(8)
        .pool_idle_timeout(Duration::from_secs(32))
        .redirect(reqwest::redirect::Policy::limited(3));

    let mut default_headers = reqwest::header::HeaderMap::new();

    for (key, value) in &config.headers {
        let header_name = reqwest::header::HeaderName::from_bytes(key.as_bytes())?;
        let header_value = reqwest::header::HeaderValue::from_str(value)?;
        default_headers.insert(header_name, header_value);
    }

    client_builder = client_builder.default_headers(default_headers);

    Ok(client_builder.build()?)
}

// WORDLIST STREAMING

async fn stream_wordlist(file_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let file = AsyncFile::open(file_path).await?;
    let reader = AsyncBufReader::new(file);
    let mut lines = reader.lines();
    let mut wordlist = Vec::new();

    while let Some(line) = lines.next_line().await? {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            wordlist.push(trimmed.to_string());
        }
    }

    if wordlist.is_empty() {
        return Err("Wordlist is empty or contains no valid entries".into());
    }

    Ok(wordlist)
}

// URL CONSTRUCTION

fn construct_url(base_url: &str, word: &str, mode: FuzzMode) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let base = Url::parse(base_url)?;

    match mode {
        FuzzMode::Subdomain => {
            let host = base.host_str().ok_or("Invalid host in URL")?;
            let new_host = if let Some(stripped) = host.strip_prefix("www.") {
                format!("{}.{}", word, stripped)
            } else {
                format!("{}.{}", word, host)
            };

            let mut new_url = base.clone();
            new_url.set_host(Some(&new_host))?;
            Ok(new_url.to_string())
        }

        FuzzMode::Directory => {
            let path_to_join = if word.starts_with('/') {
                word.to_string()
            } else {
                format!("/{}", word)
            };

            Ok(format!("{}{}", base_url.trim_end_matches('/'), path_to_join))
        }
    }
}

// HTTP REQUEST EXECUTION

async fn make_request(
    client: &Client,
    url: String,
    config: &Config,
    user_agent: &str,
) -> Result<(String, StatusCode, u64), Box<dyn std::error::Error + Send + Sync>> {
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        let mut request_builder = client.get(&url).header("User-Agent", user_agent);

        if !config.cookies.is_empty() {
            let cookie_string = config.cookies
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("; ");
            request_builder = request_builder.header("Cookie", cookie_string);
        }

        match timeout(config.timeout, request_builder.send()).await {
            Ok(Ok(response)) => {
                let status = response.status();
                let content_length = response.content_length().unwrap_or(0);
                return Ok((url, status, content_length));
            }
            Ok(Err(e)) => last_error = Some(e.into()),
            Err(_) => last_error = Some("Request timed out".to_string().into()),
        }

        if attempt < config.max_retries {
            tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
        }
    }

    Err(last_error.unwrap_or_else(|| "All retry attempts failed".to_string().into()))
}

// BRUTE FORCE ENGINE

async fn brute_force(
    config: Config,
    wordlist: Vec<String>,
    modes: Vec<FuzzMode>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = Arc::new(build_http_client(&config)?);
    let semaphore = Arc::new(Semaphore::new(config.concurrent_requests));
    let config = Arc::new(config);

    let pause_flag = Arc::new(AtomicBool::new(false));
    let resume_notify = Arc::new(Notify::new());

    {
        let pause_flag = Arc::clone(&pause_flag);
        let resume_notify = Arc::clone(&resume_notify);
        let config = Arc::clone(&config);
        tokio::spawn(async move {
            let mut system = System::new_all();
            loop {
                system.refresh_cpu_all();
                let avg_cpu = system.global_cpu_usage();
                if avg_cpu > config.max_cpu + 4.0 {
                    pause_flag.store(true, Ordering::Release);
                } else if pause_flag.swap(false, Ordering::AcqRel) {
                    resume_notify.notify_waiters();
                }
                tokio::time::sleep(Duration::from_millis(256)).await;
            }
        });
    }

    println!("Starting brute force attack...");
    println!("Target: {}", config.target_url);
    println!("Wordlist entries: {}", wordlist.len());
    println!("Modes: {}", modes.iter().map(|m| match m {
        FuzzMode::Directory => "Directory",
        FuzzMode::Subdomain => "Subdomain",
    }).collect::<Vec<_>>().join(", "));
    println!("Concurrent requests: {}", config.concurrent_requests);
    println!("Valid status codes: {:?}", VALID_STATUS_CODES);
    println!("{}", "─".repeat(64));

    let mut handles = Vec::new();

    for word in wordlist.into_iter() {
        let selected_modes: &[FuzzMode] = if word.contains('/') {
            &[FuzzMode::Directory]
        } else {
            &modes
        };
        for mode in selected_modes {
            let mode = *mode;
            let client = Arc::clone(&client);
            let config = Arc::clone(&config);
            let semaphore = Arc::clone(&semaphore);
            let pause_flag = Arc::clone(&pause_flag);
            let resume_notify = Arc::clone(&resume_notify);
            let word = word.clone();
            let user_agent = USER_AGENTS[rand::rng().random_range(0..USER_AGENTS.len())];

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                while pause_flag.load(Ordering::Relaxed) {
                    resume_notify.notified().await;
                }

                match construct_url(&config.target_url, &word, mode) {
                    Ok(url) => {
                        match make_request(&client, url.clone(), &config, user_agent).await {
                            Ok((final_url, status, content_length)) => {
                                if VALID_STATUS_CODES.contains(&status.as_u16()) {
                                    println!("[{}] {} - {} bytes", status.as_u16(), final_url, content_length);
                                }
                            }
                            Err(e) => {
                                eprintln!("Error requesting {}: {}", url, e);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error constructing URL for '{}': {}", word, e);
                    }
                }
            });

            handles.push(handle);
        }
    }

    for handle in handles {
        let _ = handle.await;
    }

    println!("{}", "─".repeat(64));
    println!("Bruteforcing completed!");

    Ok(())
}
// CLI

fn build_cli() -> Command {
    Command::new("reconcrab")
        .version("1.0.0")
        .about("A concurrent web directory and subdomain brute force tool")
        .arg_required_else_help(true)
        .arg(
            Arg::new("target")
                .short('t')
                .long("target")
                .value_name("URL")
                .help("Target URL or domain (e.g., https://example.com or https://sub.example.com)")
                .required(true)
        )
        .arg(
            Arg::new("wordlist")
                .short('w')
                .long("wordlist")
                .visible_alias("wordl")
                .value_name("FILE")
                .help("Path to wordlist file (newline separated)")
                .required(true)
        )
        .arg(
            Arg::new("concurrent")
                .short('c')
                .long("concurrent")
                .value_name("NUMBER")
                .help("Number of concurrent requests (default: 5000000)")
                .value_parser(clap::value_parser!(usize))
        )
        .arg(
            Arg::new("max_cpu")
                .long("max_cpu")
                .visible_alias("cpu")
                .value_name("PERCENT")
                .help("Maximum allowed CPU usage before blocking new requests (default: 50)")
                .value_parser(clap::value_parser!(f32))
        )
        .arg(
            Arg::new("directory")
                .short('d')
                .long("directory")
                .visible_alias("dir")
                .help("Enable directory brute-forcing (default: true if none specified)")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("subdomain")
                .short('s')
                .long("subdomain")
                .visible_alias("subd")
                .help("Enable subdomain brute-forcing")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("headers")
                .short('H')
                .long("header")
                .value_name("HEADER")
                .help("Custom headers in format 'Key: Value' (can be used multiple times)")
                .action(clap::ArgAction::Append)
        )
        .arg(
            Arg::new("cookies")
                .short('C')
                .long("cookie")
                .value_name("COOKIE")
                .help("Cookies in format 'name: value' (can be used multiple times)")
                .action(clap::ArgAction::Append)
        )
}

// MAIN FUNCTION

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let matches = build_cli().get_matches();

    let target_url = matches.get_one::<String>("target").unwrap().clone();
    let wordlist_file = matches.get_one::<String>("wordlist").unwrap().clone();
    let concurrent_requests = matches.get_one::<usize>("concurrent").copied();
    let directory_mode = matches.get_flag("directory");
    let subdomain_mode = matches.get_flag("subdomain");
    let max_cpu = matches.get_one::<f32>("max_cpu").copied();
    let modes = if !directory_mode && !subdomain_mode {
        vec![FuzzMode::Directory]
    } else {
        let mut m = Vec::new();
        if directory_mode {
            m.push(FuzzMode::Directory);
        }
        if subdomain_mode {
            m.push(FuzzMode::Subdomain);
        }
        m
    };
    let headers = matches.get_many::<String>("headers")
        .map(|v| v.cloned().collect::<Vec<_>>());
    let cookies = matches.get_many::<String>("cookies")
        .map(|v| v.cloned().collect::<Vec<_>>());

    let config = Config::new(target_url, wordlist_file.clone(), concurrent_requests, headers, cookies, max_cpu)?;

    println!("Loading wordlist from: {}", wordlist_file);
    let wordlist = stream_wordlist(&wordlist_file).await?;

    brute_force(config, wordlist, modes).await?;

    Ok(())
}

