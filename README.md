# ReconCrab

`ReconCrab` is a high-performance, asynchronous CLI tool built in Rust for brute-forcing web directories and subdomains. Designed for security researchers and penetration testers, this tool leverages massive concurrency and customization to scan large wordlists efficiently. It can fuzz both directories and subdomains using user-specified headers, cookies, and request configurations.

Whether you’re enumerating hidden endpoints or uncovering shadow subdomains, `ReconCrab` is built to deliver speed and control with minimal configuration overhead.

---

## Features

* **Blazing Fast** – Built with `tokio`, handles millions of concurrent requests efficiently.
* **Dual Fuzzing Modes** – Supports **directory** and **subdomain** brute-forcing simultaneously.
* **Highly Configurable** – Add custom headers and cookies for authenticated or complex targets.
* **Smart Request Handling** – Retries, timeouts, user-agent rotation, and status code filtering.
* **Wordlist Streaming** – Reads large files line-by-line without memory overhead.
* **Status Code Filtering** – Reports only valid/interesting HTTP responses.

---

## Installation

### From Prebuilt Releases

1. Visit the [Releases](https://github.com/dhr412/reconcrab/releases) page.
2. Download the binary for your platform.
3. Make it executable:

   ```bash
   chmod +x reconcrab-*
   ```

4. Run it with:

   ```bash
   reconcrab --help
   ```

### Compiling from Source

Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed:

```bash
git clone https://github.com/dhr412/reconcrab.git
cd reconcrab
cargo build --release
./target/release/reconcrab --help
```

---

## ⚙️ Usage

```bash
reconcrab -t <target_url> -w <wordlist_file> [OPTIONS]
```

### Required Flags

* `-t`, `--target`: Base URL or domain (e.g. `https://example.com`)
* `-w`, `--wordlist`: Path to newline-separated wordlist file

### Optional Flags

* `-c`, `--concurrent`: Number of concurrent requests (default: 5000000)
* `-H`, `--header`: Custom header(s) (`Key: Value`, can be repeated)
* `-C`, `--cookie`: Cookie(s) (`name: value`, can be repeated)
* `-h`, `--help`: Show help message and exit

### Example

```bash
reconcrab -t https://example.com -w paths.txt -c 100 -H "Authorization: Bearer TOKEN" -C "sessionid: abc123"
```

---

## 🧬 How It Works

1. **Initialization**:

   * Parses CLI input using `clap`.
   * Builds a `reqwest` HTTP client with optional headers/cookies.
   * Loads wordlist line-by-line using async file streaming.

2. **Brute Forcing**:

   * For each word, constructs two URLs: one for directory (`https://example.com/word`) and one for subdomain (`https://word.example.com`).
   * Sends asynchronous requests with randomized user agents.
   * Filters and prints only those responses with a status code in a predefined valid set.

3. **Concurrency**:

   * Uses `tokio::Semaphore` to limit concurrency.
   * Implements retry logic with exponential backoff and timeout enforcement for reliability.

4. **Output**:

   * Displays successful hits with status code and content length.
   * Errors and retries are logged quietly unless debugging is needed.

---

## 📜 License

This project is open-sourced under the MIT license. Feel free to fork, contribute, or submit issues!

