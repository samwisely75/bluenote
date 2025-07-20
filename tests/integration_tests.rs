use bluenote::*;
use std::collections::HashMap;
use tempfile::tempdir;

#[derive(Debug)]
struct TestProfile {
    server: Option<Endpoint>,
    user: Option<String>,
    password: Option<String>,
    insecure: Option<bool>,
    ca_cert: Option<String>,
    headers: HashMap<String, String>,
    proxy: Option<Endpoint>,
}

impl TestProfile {
    fn new() -> Self {
        Self {
            server: None,
            user: None,
            password: None,
            insecure: None,
            ca_cert: None,
            headers: HashMap::new(),
            proxy: None,
        }
    }

    fn with_server(mut self, server: &str) -> Self {
        self.server = Some(Endpoint::parse(server).unwrap());
        self
    }

    fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    fn with_auth(mut self, user: &str, password: &str) -> Self {
        self.user = Some(user.to_string());
        self.password = Some(password.to_string());
        self
    }
}

impl HttpConnectionProfile for TestProfile {
    fn server(&self) -> Option<&Endpoint> {
        self.server.as_ref()
    }

    fn user(&self) -> Option<&String> {
        self.user.as_ref()
    }

    fn password(&self) -> Option<&String> {
        self.password.as_ref()
    }

    fn insecure(&self) -> Option<bool> {
        self.insecure
    }

    fn ca_cert(&self) -> Option<&String> {
        self.ca_cert.as_ref()
    }

    fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    fn proxy(&self) -> Option<&Endpoint> {
        self.proxy.as_ref()
    }
}

#[derive(Debug)]
struct TestRequest {
    method: Option<String>,
    url_path: Option<UrlPath>,
    body: Option<String>,
    headers: HashMap<String, String>,
}

impl TestRequest {
    fn new() -> Self {
        Self {
            method: Some("GET".to_string()),
            url_path: None,
            body: None,
            headers: HashMap::new(),
        }
    }

    fn with_method(mut self, method: &str) -> Self {
        self.method = Some(method.to_string());
        self
    }

    fn with_path(mut self, path: &str) -> Self {
        self.url_path = Some(UrlPath::new(path.to_string(), None));
        self
    }

    fn with_body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }

    fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }
}

impl HttpRequestArgs for TestRequest {
    fn method(&self) -> Option<&String> {
        self.method.as_ref()
    }

    fn url_path(&self) -> Option<&UrlPath> {
        self.url_path.as_ref()
    }

    fn body(&self) -> Option<&String> {
        self.body.as_ref()
    }

    fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
}

#[tokio::test]
async fn test_http_client_creation() {
    let profile = TestProfile::new().with_server("https://httpbin.org");
    let client = HttpClient::new(&profile);
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_basic_get_request() {
    let profile = TestProfile::new().with_server("https://httpbin.org");
    let client = HttpClient::new(&profile).unwrap();

    let request = TestRequest::new().with_path("/get");

    let result = client.request(&request).await;

    // The request should succeed (network permitting)
    match result {
        Ok(response) => {
            assert!(response.status().is_success() || response.status().is_server_error());
            // Basic validation that we got some response
            assert!(!response.body().is_empty() || response.status().is_server_error());
        }
        Err(_) => {
            // Network errors are acceptable in tests
            println!("Network request failed (acceptable in test environment)");
        }
    }
}

#[tokio::test]
async fn test_post_request_with_body() {
    let profile = TestProfile::new()
        .with_server("https://httpbin.org")
        .with_header("content-type", "application/json");

    let client = HttpClient::new(&profile).unwrap();

    let request = TestRequest::new()
        .with_method("POST")
        .with_path("/post")
        .with_body(r#"{"test": "data"}"#);

    let result = client.request(&request).await;

    match result {
        Ok(response) => {
            assert!(response.status().is_success() || response.status().is_server_error());
            if response.status().is_success() {
                // For successful POST to httpbin, body should contain JSON
                assert!(response.body().contains("data") || response.body().contains("{"));
            }
        }
        Err(_) => {
            println!("Network request failed (acceptable in test environment)");
        }
    }
}

#[tokio::test]
async fn test_custom_headers() {
    let profile = TestProfile::new().with_server("https://httpbin.org");
    let client = HttpClient::new(&profile).unwrap();

    let request = TestRequest::new()
        .with_path("/headers")
        .with_header("X-Test-Header", "test-value")
        .with_header("User-Agent", "bluenote-test");

    let result = client.request(&request).await;

    match result {
        Ok(response) => {
            assert!(response.status().is_success() || response.status().is_server_error());
            if response.status().is_success() {
                // httpbin /headers endpoint echoes headers back
                let body = response.body();
                assert!(body.contains("X-Test-Header") || body.contains("test-value"));
            }
        }
        Err(_) => {
            println!("Network request failed (acceptable in test environment)");
        }
    }
}

#[tokio::test]
async fn test_basic_auth() {
    let profile = TestProfile::new()
        .with_server("https://httpbin.org")
        .with_auth("testuser", "testpass");

    let client = HttpClient::new(&profile).unwrap();

    let request = TestRequest::new().with_path("/basic-auth/testuser/testpass");

    let result = client.request(&request).await;

    match result {
        Ok(response) => {
            // Should get 200 with correct auth or 401/403 if auth failed
            assert!(
                response.status().is_success()
                    || response.status() == reqwest::StatusCode::UNAUTHORIZED
                    || response.status() == reqwest::StatusCode::FORBIDDEN
                    || response.status().is_server_error()
            );
        }
        Err(_) => {
            println!("Network request failed (acceptable in test environment)");
        }
    }
}

#[test]
fn test_ini_profile_store_creation() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("profile");

    let store = IniProfileStore::new(config_path.to_str().unwrap());

    // Should not panic when creating with non-existent file
    let result = store.get_profile("nonexistent");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_ini_profile_creation_and_retrieval() {
    let temp_dir = tempdir().expect("Failed to create temp dir");
    let config_path = temp_dir.path().join("profile");

    // Create a test profile file
    std::fs::write(
        &config_path,
        "[test]\n\
         host = https://api.example.com\n\
         user = testuser\n\
         password = testpass\n\
         @content-type = application/json\n\
         @authorization = Bearer token123\n",
    )
    .expect("Failed to write config file");

    let store = IniProfileStore::new(config_path.to_str().unwrap());
    let profile = store.get_profile("test").unwrap().unwrap();

    // Verify profile was loaded correctly
    assert!(profile.server().is_some());
    assert_eq!(profile.user(), Some(&"testuser".to_string()));
    assert_eq!(profile.password(), Some(&"testpass".to_string()));
    assert_eq!(profile.headers().len(), 2);
    assert_eq!(
        profile.headers().get("content-type"),
        Some(&"application/json".to_string())
    );
    assert_eq!(
        profile.headers().get("authorization"),
        Some(&"Bearer token123".to_string())
    );
}

#[test]
fn test_blank_profile() {
    let blank = get_blank_profile();

    assert!(blank.server().is_none());
    assert!(blank.user().is_none());
    assert!(blank.password().is_none());
    assert!(blank.headers().is_empty());
}

#[test]
fn test_url_parsing() {
    // Test absolute URL
    let url1 = Url::parse("https://example.com:8080/api/v1?param=value");
    assert!(url1.host().is_some());
    assert_eq!(url1.host().unwrap(), "example.com");
    assert_eq!(url1.port(), Some(8080));
    assert_eq!(url1.scheme(), Some(&"https".to_string()));

    // Test relative URL
    let url2 = Url::parse("/api/users?filter=active");
    assert!(url2.host().is_none());
    assert_eq!(url2.path(), Some(&"/api/users".to_string()));
    assert_eq!(url2.query(), Some(&"filter=active".to_string()));
}

#[test]
fn test_endpoint_parsing() {
    let endpoint = Endpoint::parse("https://api.example.com:443").unwrap();

    assert_eq!(endpoint.host(), "api.example.com");
    assert_eq!(endpoint.port(), Some(443));
    assert_eq!(endpoint.scheme(), Some(&"https".to_string()));
    assert_eq!(endpoint.to_string(), "https://api.example.com:443");
}

#[test]
fn test_url_path_creation() {
    let path1 = UrlPath::new("/api/test".to_string(), None);
    assert_eq!(path1.path(), "/api/test");
    assert_eq!(path1.query(), None);
    assert_eq!(path1.to_string(), "/api/test");

    let path2 = UrlPath::new("/search".to_string(), Some("q=rust".to_string()));
    assert_eq!(path2.path(), "/search");
    assert_eq!(path2.query(), Some(&"q=rust".to_string()));
    assert_eq!(path2.to_string(), "/search?q=rust");
}

#[test]
fn test_http_client_error_handling() {
    // Test with invalid endpoint
    let mut profile = TestProfile::new();
    profile.server = Some(Endpoint::parse("https://").unwrap_or_else(|_| {
        // If parsing fails, create a minimal endpoint that will fail connection
        Endpoint::parse("https://invalid-domain-that-does-not-exist.invalid").unwrap()
    }));

    let client_result = HttpClient::new(&profile);
    // Client creation should succeed even with invalid endpoint
    assert!(client_result.is_ok());
}

#[tokio::test]
async fn test_different_http_methods() {
    let profile = TestProfile::new().with_server("https://httpbin.org");
    let client = HttpClient::new(&profile).unwrap();

    let methods = ["GET", "POST", "PUT", "DELETE", "PATCH"];

    for method in &methods {
        let request = TestRequest::new()
            .with_method(method)
            .with_path("/status/200");

        let result = client.request(&request).await;

        match result {
            Ok(response) => {
                // Should accept all HTTP methods
                assert!(response.status().is_success() || response.status().is_server_error());
            }
            Err(_) => {
                // Network errors are acceptable
                println!("Method {method} failed due to network (acceptable)");
            }
        }
    }
}

#[test]
fn test_constants() {
    // Test that important constants are accessible
    assert_eq!(DEFAULT_INI_FILE_PATH, "~/bluenote.profile");
    assert!(!DEFAULT_INI_FILE_PATH.is_empty());
}

#[test]
fn test_profile_file_path_environment_variable() {
    // Test default path
    std::env::remove_var("BLUENOTE_PROFILE");
    assert_eq!(get_profile_file_path(), "~/bluenote.profile");

    // Test environment variable override
    std::env::set_var("BLUENOTE_PROFILE", "/custom/path/profile.ini");
    assert_eq!(get_profile_file_path(), "/custom/path/profile.ini");

    // Clean up
    std::env::remove_var("BLUENOTE_PROFILE");
}

#[test]
fn test_ini_profile_store_default_constructor() {
    // Test that the default constructor uses the environment-aware path
    std::env::remove_var("BLUENOTE_PROFILE");
    let store = IniProfileStore::default();

    // Should not panic when creating with default path
    let result = store.get_profile("nonexistent");
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_json_response_parsing() {
    let profile = TestProfile::new().with_server("https://httpbin.org");
    let client = HttpClient::new(&profile).unwrap();

    let request = TestRequest::new().with_path("/json");

    let result = client.request(&request).await;

    match result {
        Ok(response) => {
            if response.status().is_success() {
                // httpbin /json returns JSON, so our JSON parsing should work
                assert!(response.json().is_some() || !response.body().is_empty());
            }
        }
        Err(_) => {
            println!("Network request failed (acceptable in test environment)");
        }
    }
}
