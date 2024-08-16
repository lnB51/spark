use anyhow::bail;
use bot_api::Esp32Api;
use config::create_config;
use esp_idf_svc::{eventloop::EspSystemEventLoop, sys::nvs_flash_init};
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::delay::FreeRtos;
use log::info;
use frankenstein::{
     GetUpdatesParams, SendMessageParams, TelegramApi,
};
use wifi::connect_wifi;

mod bot_api;
mod wifi;
mod config;

fn main() -> anyhow::Result<()>{
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Set up peripherals and display
    let sysloop = EspSystemEventLoop::take()?;
    let peripherals = Peripherals::take().unwrap();
    let mut led = PinDriver::output(peripherals.pins.gpio8)?;
    led.set_high()?; // Turn off the LED and 8 GPIO on board (yep, it's strange that the signals are inverted on the esp32c3 super mini)

    // Load configuration settings
    let config = create_config();

    info!(
        "About to initialize WiFi (SSID: {}, PASS: {})",
        config.wifi_ssid, config.wifi_pass
    );
    unsafe { nvs_flash_init() };

    // Connect to Wi-Fi
    let _wifi = match connect_wifi(
        config.wifi_ssid,
        config.wifi_pass,
        peripherals.modem,
        sysloop,
    ) {
        Ok(inner) => inner,
        Err(err) => {
            bail!("Could not connect to Wi-Fi network: {:?}", err)
        }
    };
    
    // Initialize the Telegram API with the bot token from config
    let api = Esp32Api::new(config.bot_token);

    // Send a startup message to the bot owner (indicates that the board is turned on and ready to work)
    api.send_message(
        &SendMessageParams::builder()
            .chat_id(config.bot_owner_id)
            .text("Starting  🔌")
            .build(),
    )
    .ok();

    // Fetch the latest updates from the Telegram bot
    let updates = api
        .get_updates(&GetUpdatesParams::builder().limit(1u32).offset(-1).build())
        .unwrap();

    // Determine the initial offset for polling updates
    let mut offset = if let Some(update) = updates.result.first() {
        update.update_id + 1
    } else {
        0
    };

    // Main loop to continuously check for new messages
    loop {
        let updates = api
            .get_updates(
                &GetUpdatesParams::builder()
                    .timeout(120u32) // Set the timeout for long polling
                    .limit(1u32)
                    .offset(offset)
                    .build(),
            )
            .unwrap();

        // Iterate over each update and handle commands
        for update in updates.result {
            offset = update.update_id + 1;

            if let frankenstein::UpdateContent::Message(message) = update.content {
                info!(
                    "message id {} from chat {}",
                    message.message_id, message.chat.id
                );

                // Handle specific commands based on the message text
                match message.text.unwrap_or_default().as_str() {
                    "/poweron" => {
                        
                        // Check if a message has been sent from the owner
                        if message.chat.id != config.bot_owner_id {
                            continue;
                        }

                        // Powers LED and 8 GPIO for sending startup signal to PC (it takes 1 second, which is enough to turn it on)
                        led.set_low()?;
                        FreeRtos::delay_ms(1000);
                        led.set_high()?;

                        // Send a message to the owner chat that the computer is on 
                        let _ = api.send_message(
                            &SendMessageParams::builder()
                                .chat_id(config.bot_owner_id)
                                .text("Power on 🔋")
                                .build());
                    }
                    "/poweroff" => {

                        // Check if a message has been sent from the owner
                        if message.chat.id != config.bot_owner_id {
                            continue;
                        }

                        // Powers LED and 8 GPIO for sending poweroff signal to PC 
                        // (it takes 1 second, which is enough to turn it off, works only if PC already powered, else works same as power on)
                        // PS: I know that it's the same as power on, but I was to lazy to remove it from bot and from here
                        led.set_low()?;
                        FreeRtos::delay_ms(1000);
                        led.set_high()?;

                        // Send a message to the owner chat that the computer is off
                        let _ = api.send_message(
                            &SendMessageParams::builder()
                                .chat_id(config.bot_owner_id)
                                .text("Power off 🪫")
                                .build());
                    }
                    "/reboot" => {

                        // Check if a message has been sent from the owner
                        if message.chat.id != config.bot_owner_id {
                            continue;
                        }

                        // Powers LED and 8 GPIOs to send a power-off signal to the PC, and then send a 1-second pulse to turn it on
                        led.set_low()?;
                        FreeRtos::delay_ms(5000);
                        led.set_high()?;
                        FreeRtos::delay_ms(3000);
                        led.set_low()?;
                        FreeRtos::delay_ms(1000);
                        led.set_high()?;

                        // Send a message to the owner chat that the computer is rebooted
                        let _ = api.send_message(
                            &SendMessageParams::builder()
                                .chat_id(config.bot_owner_id)
                                .text("Rebooting 🪫-> 🔋")
                                .build());
                    }
                    _ => ()
                }
            }
        }
    }
}