# httpc-profiles

A flexible HTTP client library with profile-based configuration support for Rust.

## Features

- **Profile-based configuration**: Store connection settings, headers, and authentication in profiles
- **Multiple authentication methods**: Basic auth, custom headers, certificate-based auth  
- **Response decoding**: Automatic decompression (gzip, deflate, zstd) and encoding detection
- **Proxy support**: HTTP and HTTPS proxy configuration
- **TLS configuration**: Custom CA certificates and insecure mode support
- **Clean abstractions**: Well-defined traits for building HTTP clients
- **INI file support**: Load profiles from standard INI configuration files

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
httpc-profiles = { version = "0.2.0", features = ["ini-profiles"] }
```

## Example Usage

### Loading a Profile and Making Requests

```rust
use httpc_profiles::{HttpClient, IniProfileStore, HttpRequestArgs, UrlPath};
use std::collections::HashMap;

// Define your request
struct MyRequest {
    method: String,
    path: UrlPath,
    body: Option<String>,
    headers: HashMap<String, String>,
}

impl HttpRequestArgs for MyRequest {
    fn method(&self) -> Option<&String> { Some(&self.method) }
    fn url_path(&self) -> Option<&UrlPath> { Some(&self.path) }
    fn body(&self) -> Option<&String> { self.body.as_ref() }
    fn headers(&self) -> &HashMap<String, String> { &self.headers }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load a profile from an INI file
    let store = IniProfileStore::new("profiles.ini");
    let profile = store.get_profile("api")?.unwrap();

    // Create HTTP client with the profile
    let client = HttpClient::new(&profile)?;

    // Create your request
    let request = MyRequest {
        method: "GET".to_string(),
        path: UrlPath::new("/users"),
        body: None,
        headers: HashMap::new(),
    };

    // Make the request
    let response = client.request(&request).await?;
    println!("Status: {}", response.status());
    println!("Body: {}", response.body());

    Ok(())
}
```

### Profile Configuration (INI Format)

Create a `profiles.ini` file:

```ini
[api]
host = https://api.example.com
@content-type = application/json
@accept = application/json
@authorization = Bearer your-token-here

[local]
host = http://localhost:8080
@content-type = application/json
insecure = true

[staging]
host = https://staging-api.example.com
user = username
password = secret
ca_cert = /path/to/cert.pem
proxy = http://proxy.company.com:8080
```

### Configuration Options

Profile settings support:

- **Connection**: `host`, `insecure`, `ca_cert`, `proxy`
- **Authentication**: `user`, `password` (for Basic Auth)
- **Headers**: Prefix with `@` (e.g., `@authorization`, `@content-type`)
- **TLS**: `insecure = true` to skip certificate verification
- **Proxy**: `proxy = http://proxy-server:port`

## API Reference

### Core Traits

- **`HttpConnectionProfile`**: Defines connection settings (host, auth, TLS, proxy)
- **`HttpRequestArgs`**: Defines request parameters (method, URL, body, headers)

### Main Types

- **`HttpClient`**: The main HTTP client for making requests
- **`HttpResponse`**: Response object with status, headers, and body
- **`IniProfile`**: Profile implementation for INI-based configuration
- **`IniProfileStore`**: Manager for loading profiles from INI files

### URL and Endpoint Types

- **`Endpoint`**: Represents a server endpoint (scheme, host, port)
- **`Url`**: Complete URL with endpoint and path
- **`UrlPath`**: URL path component with query parameters

## Response Decoding

The library automatically handles:

- **Compression**: gzip, deflate, zstd decompression
- **Encoding**: UTF-8, Latin-1, and other character encodings
- **JSON parsing**: Automatic JSON deserialization when appropriate
- **Error handling**: Graceful fallbacks for malformed content

## Cargo Features

- `ini-profiles` (default): Enable INI file profile support
- `json-profiles`: Enable JSON profile configuration (future feature)

## License

Licensed under the Elastic License 2.0. See LICENSE file for details.

## Contributing

This library is extracted from the [blueline](https://github.com/samwisely75/blueline) HTTP client project.
Contributions are welcome! Please open issues and pull requests on the main repository.
