use log::info;
#[derive(Debug)]
#[toml_cfg::toml_config] // Macro to generate configuration struct from a TOML file
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str, // WiFi SSID
    #[default("")]
    wifi_pass: &'static str, // WiFi password
    #[default("")]
    bot_token: &'static str, // Bot token for authentication
    #[default(0)]
    bot_owner_id: i64, // Bot owner ID, default is 0
}

pub fn create_config() -> Config {
    info!("got config {:#?}", CONFIG); // Log the loaded configuration

    // Check if both WiFi SSID and password are empty, and panic if they are
    if CONFIG.wifi_ssid.is_empty() && CONFIG.wifi_pass.is_empty() {
        panic!("WiFi SSID and Password are not set");
    }

    CONFIG // Return the configuration
}