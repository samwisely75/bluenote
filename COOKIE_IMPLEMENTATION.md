# Cookie Store Implementation Plan for Bluenote

## Overview

This document outlines the planned implementation of persistent cookie storage in bluenote using reqwest's cookie store features.

## Architecture Design

### Cookie Store Per Profile

Each profile will maintain its own isolated cookie store to prevent session conflicts between different environments (dev/staging/prod) or APIs.

### File Structure

```
~/.blueline/cookies/
├── {profile_name}/
│   ├── {hashed_username}.json
│   └── ...
└── ...
```

- Profile name is used directly in the path
- Username is hashed using SHA256 for privacy
- JSON format for human readability and debugging

## Implementation Points

### 1. Integration in `build_client()` Function

Location: `src/http.rs::build_client()`

```rust
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};
use std::sync::Arc;

fn build_client(args: &impl HttpConnectionProfile) -> Result<Client> {
    // Load or create cookie store for this profile
    let cookie_store = load_or_create_cookie_store(
        args.profile_name(),
        args.user()
    )?;
    
    let cookie_store = Arc::new(CookieStoreMutex::new(cookie_store));
    
    let mut builder = Client::builder()
        .cookie_provider(Arc::clone(&cookie_store))
        // ... existing configuration
    
    // ... rest of existing build_client logic
}
```

### 2. Cookie Store Management

```rust
fn load_or_create_cookie_store(
    profile_name: &str,
    username: Option<&String>
) -> Result<CookieStore> {
    let path = get_cookie_store_path(profile_name, username)?;
    
    if path.exists() {
        // Load existing cookies
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        CookieStore::load_json(reader)
            .map_err(|e| anyhow!("Failed to load cookie store: {}", e))
    } else {
        // Create new cookie store
        Ok(CookieStore::new(None))
    }
}

fn save_cookie_store(
    store: &CookieStore,
    profile_name: &str,
    username: Option<&String>
) -> Result<()> {
    let path = get_cookie_store_path(profile_name, username)?;
    
    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Atomic write: write to temp file, then rename
    let temp_path = path.with_extension("tmp");
    let file = File::create(&temp_path)?;
    let mut writer = BufWriter::new(file);
    
    store.save_json(&mut writer)
        .map_err(|e| anyhow!("Failed to save cookie store: {}", e))?;
    
    writer.flush()?;
    fs::rename(temp_path, path)?;
    
    // Set file permissions to 0600 (owner read/write only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&path, perms)?;
    }
    
    Ok(())
}
```

### 3. Path Generation

```rust
use sha2::{Sha256, Digest};

fn get_cookie_store_path(
    profile_name: &str,
    username: Option<&String>
) -> Result<PathBuf> {
    let mut path = dirs::home_dir()
        .ok_or_else(|| anyhow!("Could not determine home directory"))?;
    
    path.push(".blueline");
    path.push("cookies");
    path.push(profile_name);
    
    // Hash username for privacy
    let filename = match username {
        Some(user) => {
            let mut hasher = Sha256::new();
            hasher.update(user.as_bytes());
            format!("{:x}.json", hasher.finalize())
        }
        None => "default.json".to_string(),
    };
    
    path.push(filename);
    Ok(path)
}
```

## Thread Safety Considerations

### Shared Cookie Store with Clone

When `HttpClient` is cloned:
- The `reqwest::Client` contains `Arc<CookieStoreMutex>`
- All clones share the same cookie store instance
- Updates from any clone are visible to all others
- Thread-safe due to internal mutex

### Benefits
- Consistent session state across parallel requests
- Automatic cookie synchronization
- No duplicate login requests needed

## File Persistence Strategy

### Save Timing Options

1. **After Each Request** (Recommended initially)
   - Pro: Maximum safety, no cookie loss
   - Con: More I/O operations
   - Implementation: Hook into response handling

2. **Periodic Save** (Future optimization)
   - Pro: Reduced I/O
   - Con: Risk of losing recent cookies
   - Implementation: Background timer task

3. **On Shutdown** (Supplementary)
   - Pro: Ensures final state is saved
   - Con: Risk if app crashes
   - Implementation: Drop trait or explicit cleanup

### File Locking

For multiple process safety:
- Use atomic writes (write to temp, rename)
- Consider using `flock` on Unix systems
- Accept that simultaneous writes may occur (last write wins)

## Privacy & Security

### File Permissions
- Set to 0600 (owner read/write only) on Unix
- Store in user-specific directory

### Username Hashing
- Use SHA256 for consistent, one-way hashing
- Prevents username exposure in filesystem

### Cookie Isolation
- Separate stores per profile
- No cross-profile cookie leakage
- Easy to clear per-profile sessions

## Dependencies to Add

```toml
[dependencies]
reqwest = { version = "0.12", features = ["cookies"] }
reqwest_cookie_store = "0.8"
cookie_store = "0.21"
sha2 = "0.10"
dirs = "5.0"
```

## Migration Path

1. Phase 1: Basic cookie store with file persistence
2. Phase 2: Add save/load error recovery
3. Phase 3: Implement periodic saves
4. Phase 4: Add cookie management commands (clear, export, import)

## Testing Considerations

- Test cookie persistence across app restarts
- Test profile isolation
- Test concurrent access from multiple clones
- Test file permission settings
- Test error recovery (corrupted cookie file)

## Future Enhancements

- Cookie expiration cleanup
- Cookie viewer/editor in REPL
- Cookie import/export commands
- Per-domain cookie management
- Cookie encryption at rest