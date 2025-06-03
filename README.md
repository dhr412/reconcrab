# ReconCrab

`ReconCrab` is a high-performance, asynchronous CLI tool built in Rust for brute-forcing web directories and subdomains. This tool leverages massive concurrency and customization to scan large wordlists efficiently. It can fuzz both directories and subdomains using user-specified headers, cookies, and request configurations.

Whether you’re enumerating hidden endpoints or uncovering shadow subdomains, `ReconCrab` is built to deliver speed and control with minimal configuration overhead while keeping resource usage in check.

---

## Features

* **Blazing Fast** – Built with `tokio`, handles millions of concurrent requests efficiently.
* Resource-Aware Throttling – Dynamically pauses requests when CPU usage exceeds a configurable threshold
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

## Usage

```bash
reconcrab -t <target_url> -w <wordlist_file> [OPTIONS]
```

### Required Flags

* `-t`, `--target`: Base URL or domain (e.g. `https://example.com`)
* `-w`, `--wordlist`: Path to newline-separated wordlist file

### Optional Flags

Optional Flags

* `-d`, `--directory`, `--dir`: Enable directory brute-forcing

* `-s`, `--subdomain`, `--subd`: Enable subdomain brute-forcing

* `-c`, `--concurrent`: Number of concurrent requests (default: 500000)

* `--cpu`, `--max_cpu`: Maximum percentage of CPU to be used by the tool (default: 50%)

* `-H`, `--header`: Custom header(s) (Key: Value, can be repeated)

* `-C`, `--cookie`: Cookie(s) (name: value, can be repeated)

* `-h`, `--help`: Show help message and exit

### Example

```bash
reconcrab -t https://example.com -w paths.txt -d -s -c 100 -H "Authorization: Bearer TOKEN" -C "sessionid: abc123"
```

---

## How It Works

1. Initialization:

    * Input is parsed using the `clap` library.

    * An HTTP client is built using `reqwest`, with optional headers and cookies applied.

    * The wordlist is streamed asynchronously, reading line-by-line to reduce memory usage.

2. Brute Forcing:

    * URLs are constructed based on the selected mode (directory or subdomain).

    * Requests are sent asynchronously, with user agents randomized for each.

    * Responses are filtered based on a predefined set of valid status codes.

3. Concurrency:

    * Semaphore is used to limit the number of concurrent requests.

    * Retry logic with exponential backoff and request timeouts is implemented for robustness.

    * Halts requests if cpu usage exceeds maximum allowed usage

4. Output:

    * Successful responses are displayed with status code and content length.

    * Errors and retries are handled quietly.

---

## License

This project is open-sourced under the MIT license. Feel free to fork, contribute, or submit issues!
