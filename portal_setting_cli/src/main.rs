use anyhow::Result;
use portal_setting::SettingsPortal;
use zbus::Connection;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting XDG Portal Settings Service...");

    // Create the settings portal
    let portal = SettingsPortal::new();

    // Connect to session bus
    let connection = Connection::session().await?;

    // Request the well-known name
    connection
        .request_name("org.freedesktop.impl.portal.Settings")
        .await?;

    println!("Service registered at org.freedesktop.impl.portal.Settings");

    // Serve the interface at the standard path
    connection
        .object_server()
        .at("/org/freedesktop/portal/desktop", portal)
        .await?;

    println!("Service is ready at /org/freedesktop/portal/desktop");
    println!("Press Ctrl+C to stop the service");

    // Keep the service running
    std::future::pending::<()>().await;

    Ok(())
}
