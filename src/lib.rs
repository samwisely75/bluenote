//! # HTTP Profiles Client Library
//!
//! A flexible HTTP client library with profile-based configuration support.
//! This library provides a clean interface for making HTTP requests with
//! configuration profiles that can be stored in INI files.
//!
//! ## Features
//!
//! - **Profile-based configuration**: Store connection settings, headers, and authentication in profiles
//! - **Multiple authentication methods**: Basic auth, custom headers, certificate-based auth
//! - **Response decoding**: Automatic decompression and encoding detection
//! - **Proxy support**: HTTP and HTTPS proxy configuration
//! - **TLS configuration**: Custom CA certificates and insecure mode support
//! - **Request/Response traits**: Clean abstractions for building HTTP clients
//!
//! ## Example
//!
//! ```rust,no_run
//! use bluenote::{HttpClient, IniProfile, IniProfileStore};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Load a profile from an INI file
//! let store = IniProfileStore::new("profiles.ini");
//! let profile = store.get_profile("api")?.unwrap();
//!
//! // Create HTTP client with the profile
//! let client = HttpClient::new(&profile)?;
//!
//! // Make a request (requires implementing HttpRequestArgs trait)
//! // let response = client.request(&request_args).await?;
//! # Ok(())
//! # }
//! ```

pub mod decoder;
pub mod http;
pub mod ini;
pub mod url;
pub mod utils;

// Re-export commonly used types
pub use http::{HttpClient, HttpConnectionProfile, HttpRequestArgs, HttpResponse};
pub use ini::{
    get_blank_profile, get_profile_file_path, IniProfile, IniProfileStore, DEFAULT_INI_FILE_PATH,
};
pub use url::{Endpoint, Url, UrlPath};
pub use utils::Result;
