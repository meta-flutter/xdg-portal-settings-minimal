# XDG Portal Settings Minimal

A minimal implementation of the XDG Desktop Portal Settings interface (`org.freedesktop.impl.portal.Settings`) in Rust.

## Overview

This workspace provides a D-Bus service that implements the Settings portal as specified in the [XDG Desktop Portal documentation](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html). The implementation supports all known settings with type-checking on updates and filtering on key listing.

## Workspace Structure

The project consists of three crates:

### 1. `portal_setting` (Library)

The core library that implements the D-Bus interface. Features:
- Full implementation of `org.freedesktop.impl.portal.Settings` interface
- Support for all documented settings across multiple namespaces
- Type validation on setting updates
- Namespace filtering for efficient queries
- Thread-safe settings storage using Tokio's async RwLock

### 2. `portal_setting_cli` (Executable)

A D-Bus service executable that runs the settings portal. Usage:

```bash
cargo run --bin portal-setting-service
```

The service will:
- Register at `org.freedesktop.impl.portal.Settings`
- Serve the interface at `/org/freedesktop/portal/desktop`
- Run until interrupted (Ctrl+C)

### 3. `portal_setting_client` (Test Executable)

A comprehensive test client that exercises all settings. Usage:

```bash
cargo run --bin portal-setting-client
```

The client will:
- Connect to the running service
- Read all settings with and without namespace filtering
- Verify individual setting reads
- Test all settings in each namespace
- Validate value types
- Report comprehensive test results

## Supported Settings

### `org.freedesktop.appearance`

| Key | Type | Valid Values | Description |
|-----|------|--------------|-------------|
| `color-scheme` | `u32` | 0-2 | Color scheme preference (0: no preference, 1: dark, 2: light) |
| `accent-color` | `(f64, f64, f64)` | RGB tuple | Accent color as RGB values (0.0-1.0) |
| `contrast` | `u32` | 0-1 | Contrast preference (0: no preference, 1: high contrast) |

### `org.gnome.desktop.interface`

| Key | Type | Valid Values | Description |
|-----|------|--------------|-------------|
| `gtk-theme` | `string` | Any | GTK theme name |
| `icon-theme` | `string` | Any | Icon theme name |
| `cursor-theme` | `string` | Any | Cursor theme name |
| `font-name` | `string` | Any | Default font |
| `monospace-font-name` | `string` | Any | Monospace font |
| `clock-format` | `string` | `"12h"` or `"24h"` | Clock format preference |

### `org.gnome.desktop.privacy`

| Key | Type | Valid Values | Description |
|-----|------|--------------|-------------|
| `remember-recent-files` | `bool` | true/false | Whether to remember recently opened files |
| `recent-files-max-age` | `i32` | Any | Maximum age in days for recent files |

## Building

Build the entire workspace:

```bash
cargo build
```

Build in release mode:

```bash
cargo build --release
```

## Running Tests

Run the library unit tests:

```bash
cargo test -p portal_setting
```

Run integration tests (requires D-Bus session bus):

```bash
# Terminal 1: Start the service
cargo run --bin portal-setting-service

# Terminal 2: Run the client tests
cargo run --bin portal-setting-client
```

## Usage Examples

### Starting the Service

```bash
cargo run --bin portal-setting-service
```

Output:
```
Starting XDG Portal Settings Service...
Service registered at org.freedesktop.impl.portal.Settings
Service is ready at /org/freedesktop/portal/desktop
Press Ctrl+C to stop the service
```

### Running Client Tests

```bash
cargo run --bin portal-setting-client
```

The client will execute comprehensive tests and display results for:
- ReadAll operations (with and without namespace filtering)
- Individual Read operations
- All settings in each namespace
- Type verification for all values

## D-Bus Interface

### Methods

#### `Read(namespace: String, key: String) -> Variant`

Reads a single setting value.

Example:
```
gdbus call --session \
  --dest org.freedesktop.impl.portal.Settings \
  --object-path /org/freedesktop/portal/desktop \
  --method org.freedesktop.impl.portal.Settings.Read \
  "org.freedesktop.appearance" "color-scheme"
```

#### `ReadAll(namespaces: Array<String>) -> Dict<String, Dict<String, Variant>>`

Reads all settings, optionally filtered by namespaces.

Example (all settings):
```
gdbus call --session \
  --dest org.freedesktop.impl.portal.Settings \
  --object-path /org/freedesktop/portal/desktop \
  --method org.freedesktop.impl.portal.Settings.ReadAll \
  "[]"
```

Example (filtered):
```
gdbus call --session \
  --dest org.freedesktop.impl.portal.Settings \
  --object-path /org/freedesktop/portal/desktop \
  --method org.freedesktop.impl.portal.Settings.ReadAll \
  "['org.freedesktop.appearance']"
```

### Signals

#### `SettingChanged(namespace: String, key: String, value: Variant)`

Emitted when a setting value changes (implementation included but not actively used in this minimal version).

## Development

### Type Validation

The library performs strict type validation on all setting updates. Invalid types or out-of-range values will result in an error. This ensures type safety and prevents invalid configurations.

### Extensibility

Unknown settings (those not in the predefined list) are allowed for extensibility. The validation system only enforces constraints on known settings.

### Architecture

```
┌─────────────────────────────────────┐
│  portal_setting_client (testing)    │
└──────────────┬──────────────────────┘
               │ D-Bus
               ▼
┌─────────────────────────────────────┐
│  portal_setting_cli (service)       │
│  ┌───────────────────────────────┐  │
│  │  portal_setting (library)     │  │
│  │  - SettingsStore              │  │
│  │  - SettingsPortal             │  │
│  │  - Type validation            │  │
│  └───────────────────────────────┘  │
└─────────────────────────────────────┘
```

## CI/CD

The project includes GitHub Actions CI that:
1. Builds all workspace members
2. Runs unit tests
3. Runs integration tests (service + client)
4. On tag push: Builds release binaries and publishes as GitHub release

## Requirements

- Rust 1.70 or later
- D-Bus session bus (for running the service and tests)

## License

MIT

## Contributing

Contributions are welcome! Please ensure:
- All tests pass
- Code follows Rust conventions
- New settings include type validation
- Documentation is updated

## References

- [XDG Desktop Portal Specification](https://flatpak.github.io/xdg-desktop-portal/)
- [Settings Portal Documentation](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html)
- [zbus - Rust D-Bus library](https://docs.rs/zbus/)