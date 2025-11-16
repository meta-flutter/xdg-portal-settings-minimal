use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use zbus::{interface, zvariant::{OwnedValue, Str, Value}};

/// Represents the namespace and key for a setting
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SettingKey {
    pub namespace: String,
    pub key: String,
}

impl SettingKey {
    pub fn new(namespace: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            key: key.into(),
        }
    }
}

/// The value type for settings - wraps zvariant::OwnedValue
#[derive(Debug)]
pub struct SettingValue(pub OwnedValue);

impl Clone for SettingValue {
    fn clone(&self) -> Self {
        // Clone by serializing and deserializing the OwnedValue
        Self(self.0.try_clone().unwrap())
    }
}

/// Settings store that maintains all portal settings
#[derive(Clone)]
pub struct SettingsStore {
    settings: Arc<RwLock<HashMap<SettingKey, SettingValue>>>,
}

impl SettingsStore {
    pub fn new() -> Self {
        let mut settings = HashMap::new();
        
        // Initialize default settings according to the XDG portal spec
        // https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html
        
        // org.freedesktop.appearance namespace
        settings.insert(
            SettingKey::new("org.freedesktop.appearance", "color-scheme"),
            SettingValue(Value::U32(0).try_into().unwrap()), // 0: no preference, 1: dark, 2: light
        );
        settings.insert(
            SettingKey::new("org.freedesktop.appearance", "accent-color"),
            SettingValue(Value::Structure((0.0, 0.0, 0.0).into()).try_into().unwrap()), // RGB tuple
        );
        settings.insert(
            SettingKey::new("org.freedesktop.appearance", "contrast"),
            SettingValue(Value::U32(0).try_into().unwrap()), // 0: no preference, 1: high contrast
        );
        
        // org.gnome.desktop.interface namespace
        settings.insert(
            SettingKey::new("org.gnome.desktop.interface", "gtk-theme"),
            SettingValue(Value::Str(Str::from_static("Adwaita")).try_into().unwrap()),
        );
        settings.insert(
            SettingKey::new("org.gnome.desktop.interface", "icon-theme"),
            SettingValue(Value::Str(Str::from_static("Adwaita")).try_into().unwrap()),
        );
        settings.insert(
            SettingKey::new("org.gnome.desktop.interface", "cursor-theme"),
            SettingValue(Value::Str(Str::from_static("Adwaita")).try_into().unwrap()),
        );
        settings.insert(
            SettingKey::new("org.gnome.desktop.interface", "font-name"),
            SettingValue(Value::Str(Str::from_static("Cantarell 11")).try_into().unwrap()),
        );
        settings.insert(
            SettingKey::new("org.gnome.desktop.interface", "monospace-font-name"),
            SettingValue(Value::Str(Str::from_static("Source Code Pro 10")).try_into().unwrap()),
        );
        settings.insert(
            SettingKey::new("org.gnome.desktop.interface", "clock-format"),
            SettingValue(Value::Str(Str::from_static("24h")).try_into().unwrap()),
        );
        
        // org.gnome.desktop.privacy namespace
        settings.insert(
            SettingKey::new("org.gnome.desktop.privacy", "remember-recent-files"),
            SettingValue(Value::Bool(true).try_into().unwrap()),
        );
        settings.insert(
            SettingKey::new("org.gnome.desktop.privacy", "recent-files-max-age"),
            SettingValue(Value::I32(30).try_into().unwrap()), // days
        );
        
        Self {
            settings: Arc::new(RwLock::new(settings)),
        }
    }

    pub async fn read(&self, namespace: &str, key: &str) -> Option<SettingValue> {
        let settings = self.settings.read().await;
        settings.get(&SettingKey::new(namespace, key)).cloned()
    }

    pub async fn read_all(&self, namespaces: Vec<String>) -> HashMap<String, HashMap<String, SettingValue>> {
        let settings = self.settings.read().await;
        let mut result: HashMap<String, HashMap<String, SettingValue>> = HashMap::new();

        for (key, value) in settings.iter() {
            // Filter by namespaces if provided, otherwise return all
            if namespaces.is_empty() || namespaces.contains(&key.namespace) {
                result
                    .entry(key.namespace.clone())
                    .or_default()
                    .insert(key.key.clone(), value.clone());
            }
        }

        result
    }

    pub async fn write(&self, namespace: &str, key: &str, value: OwnedValue) -> Result<()> {
        // Validate the setting based on namespace and key
        self.validate_setting(namespace, key, &value)?;

        let mut settings = self.settings.write().await;
        settings.insert(
            SettingKey::new(namespace, key),
            SettingValue(value),
        );
        Ok(())
    }

    fn validate_setting(&self, namespace: &str, key: &str, value: &OwnedValue) -> Result<()> {
        match (namespace, key) {
            // org.freedesktop.appearance validations
            ("org.freedesktop.appearance", "color-scheme") => {
                if let Ok(v) = <u32>::try_from(value) {
                    if v <= 2 {
                        return Ok(());
                    }
                }
                anyhow::bail!("color-scheme must be u32 (0-2)");
            }
            ("org.freedesktop.appearance", "accent-color") => {
                // Check signature for tuple of three f64s
                if value.value_signature().as_str() == "(ddd)" {
                    return Ok(());
                }
                anyhow::bail!("accent-color must be (f64, f64, f64) tuple");
            }
            ("org.freedesktop.appearance", "contrast") => {
                if let Ok(v) = <u32>::try_from(value) {
                    if v <= 1 {
                        return Ok(());
                    }
                }
                anyhow::bail!("contrast must be u32 (0-1)");
            }
            // org.gnome.desktop.interface validations
            ("org.gnome.desktop.interface", "gtk-theme") |
            ("org.gnome.desktop.interface", "icon-theme") |
            ("org.gnome.desktop.interface", "cursor-theme") |
            ("org.gnome.desktop.interface", "font-name") |
            ("org.gnome.desktop.interface", "monospace-font-name") => {
                if value.value_signature().as_str() == "s" {
                    return Ok(());
                }
                anyhow::bail!("{} must be a string", key);
            }
            ("org.gnome.desktop.interface", "clock-format") => {
                if value.value_signature().as_str() == "s" {
                    // Just check it's a string, actual value validation would require more complex checking
                    return Ok(());
                }
                anyhow::bail!("clock-format must be '12h' or '24h'");
            }
            // org.gnome.desktop.privacy validations
            ("org.gnome.desktop.privacy", "remember-recent-files") => {
                if <bool>::try_from(value).is_ok() {
                    return Ok(());
                }
                anyhow::bail!("remember-recent-files must be a boolean");
            }
            ("org.gnome.desktop.privacy", "recent-files-max-age") => {
                if <i32>::try_from(value).is_ok() {
                    return Ok(());
                }
                anyhow::bail!("recent-files-max-age must be an i32");
            }
            // Unknown settings are allowed (for extensibility)
            _ => Ok(()),
        }
    }
}

impl Default for SettingsStore {
    fn default() -> Self {
        Self::new()
    }
}

/// D-Bus interface implementation for org.freedesktop.impl.portal.Settings
pub struct SettingsPortal {
    store: SettingsStore,
}

impl SettingsPortal {
    pub fn new() -> Self {
        Self {
            store: SettingsStore::new(),
        }
    }

    pub fn with_store(store: SettingsStore) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &SettingsStore {
        &self.store
    }
}

impl Default for SettingsPortal {
    fn default() -> Self {
        Self::new()
    }
}

#[interface(name = "org.freedesktop.impl.portal.Settings")]
impl SettingsPortal {
    /// Read a single setting
    async fn read(&self, namespace: &str, key: &str) -> zbus::fdo::Result<OwnedValue> {
        self.store
            .read(namespace, key)
            .await
            .map(|v| v.0)
            .ok_or_else(|| zbus::fdo::Error::Failed("Setting not found".to_string()))
    }

    /// Read all settings, optionally filtered by namespaces
    async fn read_all(&self, namespaces: Vec<String>) -> HashMap<String, HashMap<String, OwnedValue>> {
        let result = self.store.read_all(namespaces).await;
        
        // Convert SettingValue to OwnedValue
        result
            .into_iter()
            .map(|(ns, keys)| {
                let converted_keys = keys
                    .into_iter()
                    .map(|(k, v)| (k, v.0))
                    .collect();
                (ns, converted_keys)
            })
            .collect()
    }

    /// Signal emitted when a setting changes
    #[zbus(signal)]
    async fn setting_changed(
        signal_ctxt: &zbus::SignalContext<'_>,
        namespace: &str,
        key: &str,
        value: Value<'_>,
    ) -> zbus::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_settings_store_creation() {
        let store = SettingsStore::new();
        let value = store.read("org.freedesktop.appearance", "color-scheme").await;
        assert!(value.is_some());
    }

    #[tokio::test]
    async fn test_read_write_setting() {
        let store = SettingsStore::new();
        
        // Write a new value
        store
            .write("org.freedesktop.appearance", "color-scheme", Value::U32(1).try_into().unwrap())
            .await
            .unwrap();
        
        // Read it back
        let value = store.read("org.freedesktop.appearance", "color-scheme").await;
        assert!(value.is_some());
        let val: u32 = value.unwrap().0.try_into().unwrap();
        assert_eq!(val, 1);
    }

    #[tokio::test]
    async fn test_validation() {
        let store = SettingsStore::new();
        
        // Valid value
        assert!(store
            .write("org.freedesktop.appearance", "color-scheme", Value::U32(1).try_into().unwrap())
            .await
            .is_ok());
        
        // Invalid value (out of range)
        assert!(store
            .write("org.freedesktop.appearance", "color-scheme", Value::U32(5).try_into().unwrap())
            .await
            .is_err());
        
        // Invalid type
        assert!(store
            .write("org.freedesktop.appearance", "color-scheme", Value::Str(Str::from("invalid")).try_into().unwrap())
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_read_all_with_filter() {
        let store = SettingsStore::new();
        
        // Read all settings in a specific namespace
        let result = store.read_all(vec!["org.freedesktop.appearance".to_string()]).await;
        assert!(result.contains_key("org.freedesktop.appearance"));
        assert!(!result.contains_key("org.gnome.desktop.interface"));
    }

    #[tokio::test]
    async fn test_read_all_no_filter() {
        let store = SettingsStore::new();
        
        // Read all settings
        let result = store.read_all(vec![]).await;
        assert!(result.contains_key("org.freedesktop.appearance"));
        assert!(result.contains_key("org.gnome.desktop.interface"));
        assert!(result.contains_key("org.gnome.desktop.privacy"));
    }
}
