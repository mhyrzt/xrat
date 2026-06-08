# 07.7 Easy, P3: Desktop proxy (macOS)

**Difficulty:** Easy — half day (lower priority).

**File:** `src/app/commands/proxy/desktop.rs`

Add `#[cfg(target_os = "macos")]` path using `networksetup`:

```rust
#[cfg(target_os = "macos")]
fn run(context: &AppContext, action: &ProxyDesktopAction) -> Result<()> {
    match action {
        ProxyDesktopAction::Enable(_) => {
            let active = resolve_active_endpoints(context).await?;
            // networksetup -setwebproxy Wi-Fi <host> <port>
            // networksetup -setsecurewebproxy Wi-Fi <host> <port>
            // networksetup -setsocksfirewallproxy Wi-Fi <host> <port>
        }
        ProxyDesktopAction::Disable(_) => {
            // networksetup -setwebproxystate Wi-Fi off
            // networksetup -setsecurewebproxystate Wi-Fi off
            // networksetup -setsocksfirewallproxystate Wi-Fi off
        }
        // ...
    }
}
```

**No library needed** — just `std::process::Command` calling `networksetup`.
