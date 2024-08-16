use anyhow::{bail, Result};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripheral,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use log::info;

/// Connects to a WiFi network using the provided SSID and password.
///
/// # Arguments
///
/// * `ssid` - The SSID (name) of the WiFi network to connect to.
/// * `pass` - The password of the WiFi network.
/// * `modem` - The WiFi modem peripheral.
/// * `sysloop` - The system event loop, used for handling events.
///
/// # Result
///
/// Returns connection to network which can be used later for accessing web
pub fn connect_wifi(
    ssid: &str,
    pass: &str,
    modem: impl peripheral::Peripheral<P = esp_idf_svc::hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> Result<Box<EspWifi<'static>>> {
    let mut auth_method = AuthMethod::WPA2Personal;

    // Check if SSID is provided; if not, bail with an error
    if ssid.is_empty() {
        bail!("Missing WiFi name")
    }

    // If the password is empty, use open authentication (no password)
    if pass.is_empty() {
        auth_method = AuthMethod::None;
        info!("Wifi password is empty");
    }

    // Initialize the WiFi peripheral with the given modem and system event loop
    let mut esp_wifi = EspWifi::new(modem, sysloop.clone(), None)?;

    // Wrap the WiFi interface in a blocking handler
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;

    // Set the WiFi configuration to use the client mode
    wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;

    info!("Starting wifi...");

    // Start the WiFi interface
    wifi.start()?;

    info!("Scanning...");

    // Perform a scan to find available access points
    let ap_infos = wifi.scan()?;

    // Find the access point with the matching SSID
    let ours = ap_infos.into_iter().find(|a| a.ssid == ssid);

    // Determine the channel of the found access point, if any
    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            ssid, ours.channel
        );
        Some(ours.channel) // Use the found channel
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            ssid
        );
        None // No channel found, proceed without specifying one
    };

    // Reconfigure the WiFi client with the SSID, password, and channel
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid
            .try_into()
            .expect("SSID could not be converted to heapless String"),
        password: pass
            .try_into()
            .expect("Password could not be converted to heapless String"),
        channel,
        auth_method,
        ..Default::default()
    }))?;

    info!("Connecting wifi...");

    // Attempt to connect to the WiFi network
    wifi.connect()?;

    info!("Waiting for DHCP lease...");

    // Wait for the network interface to be ready and obtain an IP address
    wifi.wait_netif_up()?;

    // Retrieve the IP information assigned to the interface
    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    info!("Wifi DHCP info: {:?}", ip_info);

    // Return the initialized and connected WiFi interface
    Ok(Box::new(esp_wifi))
}