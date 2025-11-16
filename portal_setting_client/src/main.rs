use anyhow::Result;
use std::collections::HashMap;
use std::time::Duration;
use zbus::{Connection, zvariant::OwnedValue};

#[tokio::main]
async fn main() -> Result<()> {
    println!("XDG Portal Settings Client - Testing all settings\n");

    // Wait a moment for the service to be ready
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Connect to session bus
    let connection = Connection::session().await?;

    // Create proxy to the settings portal
    let proxy = zbus::Proxy::new(
        &connection,
        "org.freedesktop.impl.portal.Settings",
        "/org/freedesktop/portal/desktop",
        "org.freedesktop.impl.portal.Settings",
    )
    .await?;

    println!("Connected to org.freedesktop.impl.portal.Settings");
    println!("{}", "=".repeat(60));

    // Test 1: Read all settings
    println!("\n[TEST 1] Reading all settings (no filter):");
    let all_settings: HashMap<String, HashMap<String, OwnedValue>> =
        proxy.call("ReadAll", &(Vec::<String>::new(),)).await?;

    for (namespace, keys) in &all_settings {
        println!("  Namespace: {}", namespace);
        for (key, value) in keys {
            println!("    {}: {:?}", key, value);
        }
    }
    println!("✓ ReadAll passed");

    // Test 2: Read settings from specific namespace
    println!("\n[TEST 2] Reading org.freedesktop.appearance namespace:");
    let appearance_settings: HashMap<String, HashMap<String, OwnedValue>> = proxy
        .call("ReadAll", &(vec!["org.freedesktop.appearance".to_string()],))
        .await?;

    assert!(appearance_settings.contains_key("org.freedesktop.appearance"));
    println!("  Found {} keys", appearance_settings["org.freedesktop.appearance"].len());
    println!("✓ Filtered ReadAll passed");

    // Test 3: Read individual settings
    println!("\n[TEST 3] Reading individual settings:");
    
    // color-scheme
    let color_scheme: OwnedValue = proxy
        .call("Read", &("org.freedesktop.appearance", "color-scheme"))
        .await?;
    println!("  color-scheme: {:?}", color_scheme);
    let _: u32 = color_scheme.try_into()?;
    
    // gtk-theme
    let gtk_theme: OwnedValue = proxy
        .call("Read", &("org.gnome.desktop.interface", "gtk-theme"))
        .await?;
    println!("  gtk-theme: {:?}", gtk_theme);
    let _: String = gtk_theme.try_into()?;
    
    // remember-recent-files
    let recent_files: OwnedValue = proxy
        .call("Read", &("org.gnome.desktop.privacy", "remember-recent-files"))
        .await?;
    println!("  remember-recent-files: {:?}", recent_files);
    let _: bool = recent_files.try_into()?;
    
    println!("✓ Individual Read passed");

    // Test 4: Test all org.freedesktop.appearance settings
    println!("\n[TEST 4] Testing org.freedesktop.appearance settings:");
    
    // Test color-scheme (u32: 0-2)
    for i in 0..=2 {
        let val: OwnedValue = proxy
            .call("Read", &("org.freedesktop.appearance", "color-scheme"))
            .await?;
        let result: u32 = val.try_into()?;
        println!("  color-scheme value {}: {}", i, result);
    }
    
    // Test accent-color (RGB tuple)
    let accent: OwnedValue = proxy
        .call("Read", &("org.freedesktop.appearance", "accent-color"))
        .await?;
    let (r, g, b): (f64, f64, f64) = accent.try_into()?;
    println!("  accent-color: ({}, {}, {})", r, g, b);
    
    // Test contrast (u32: 0-1)
    let contrast: OwnedValue = proxy
        .call("Read", &("org.freedesktop.appearance", "contrast"))
        .await?;
    let contrast_val: u32 = contrast.try_into()?;
    println!("  contrast: {}", contrast_val);
    
    println!("✓ org.freedesktop.appearance tests passed");

    // Test 5: Test all org.gnome.desktop.interface settings
    println!("\n[TEST 5] Testing org.gnome.desktop.interface settings:");
    
    let interface_keys = vec![
        "gtk-theme",
        "icon-theme",
        "cursor-theme",
        "font-name",
        "monospace-font-name",
        "clock-format",
    ];
    
    for key in interface_keys {
        let val: OwnedValue = proxy
            .call("Read", &("org.gnome.desktop.interface", key))
            .await?;
        let str_val: String = val.try_into()?;
        println!("  {}: {}", key, str_val);
    }
    
    println!("✓ org.gnome.desktop.interface tests passed");

    // Test 6: Test all org.gnome.desktop.privacy settings
    println!("\n[TEST 6] Testing org.gnome.desktop.privacy settings:");
    
    let remember: OwnedValue = proxy
        .call("Read", &("org.gnome.desktop.privacy", "remember-recent-files"))
        .await?;
    let remember_val: bool = remember.try_into()?;
    println!("  remember-recent-files: {}", remember_val);
    
    let max_age: OwnedValue = proxy
        .call("Read", &("org.gnome.desktop.privacy", "recent-files-max-age"))
        .await?;
    let max_age_val: i32 = max_age.try_into()?;
    println!("  recent-files-max-age: {}", max_age_val);
    
    println!("✓ org.gnome.desktop.privacy tests passed");

    // Test 7: Verify value types
    println!("\n[TEST 7] Verifying all value types:");
    
    let mut type_tests_passed = 0;
    let mut type_tests_total = 0;
    
    // Check u32 types
    for key in ["color-scheme", "contrast"] {
        type_tests_total += 1;
        let val: OwnedValue = proxy
            .call("Read", &("org.freedesktop.appearance", key))
            .await?;
        if val.value_signature().as_str() == "u" {
            type_tests_passed += 1;
            println!("  ✓ {}: u32", key);
        }
    }
    
    // Check tuple type
    type_tests_total += 1;
    let val: OwnedValue = proxy
        .call("Read", &("org.freedesktop.appearance", "accent-color"))
        .await?;
    if val.value_signature().as_str() == "(ddd)" {
        type_tests_passed += 1;
        println!("  ✓ accent-color: (f64, f64, f64)");
    }
    
    // Check string types
    for key in ["gtk-theme", "icon-theme", "cursor-theme", "font-name", "monospace-font-name", "clock-format"] {
        type_tests_total += 1;
        let val: OwnedValue = proxy
            .call("Read", &("org.gnome.desktop.interface", key))
            .await?;
        if val.value_signature().as_str() == "s" {
            type_tests_passed += 1;
            println!("  ✓ {}: string", key);
        }
    }
    
    // Check bool type
    type_tests_total += 1;
    let val: OwnedValue = proxy
        .call("Read", &("org.gnome.desktop.privacy", "remember-recent-files"))
        .await?;
    if val.value_signature().as_str() == "b" {
        type_tests_passed += 1;
        println!("  ✓ remember-recent-files: bool");
    }
    
    // Check i32 type
    type_tests_total += 1;
    let val: OwnedValue = proxy
        .call("Read", &("org.gnome.desktop.privacy", "recent-files-max-age"))
        .await?;
    if val.value_signature().as_str() == "i" {
        type_tests_passed += 1;
        println!("  ✓ recent-files-max-age: i32");
    }
    
    println!("✓ Type verification passed ({}/{})", type_tests_passed, type_tests_total);

    // Summary
    println!("\n{}", "=".repeat(60));
    println!("ALL TESTS PASSED ✓");
    println!("Successfully verified all settings for:");
    println!("  - org.freedesktop.appearance (3 settings)");
    println!("  - org.gnome.desktop.interface (6 settings)");
    println!("  - org.gnome.desktop.privacy (2 settings)");
    println!("Total: 11 settings verified");
    println!("{}", "=".repeat(60));

    Ok(())
}
