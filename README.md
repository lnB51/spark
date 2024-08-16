
<p align="center">
<img src="https://github.com/lnB51/spark/blob/master/images/logo.png" width=150px alt="spark logo"/>
<br />
<h1 align="center">Spark</h1>
</p>

#### Remote starter for PC
![Version 0.1](https://img.shields.io/badge/Version%200.1-FFC832?style=for-the-badge&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-000?style=for-the-badge&logo=rust&logoColor=white)
[![MIT License](https://img.shields.io/badge/MIT%20License-004772?style=for-the-badge&logo=license&logoColor=white)](https://github.com/lnB51/spark/blob/master/LICENSE)
<p>
  This project was designed to solve the problem of arranging a remote workplace. If there were no problems with remote control tools, how to run a computer remotely with minimal costs is a difficult question.
</p>

## What You’ll Need:
    
  - An ESP32-C3 board ⚙️
  - Your trusty PC or laptop 🐱‍💻
  - A cup of tea or coffee (your choice!) ☕
  - A good mood 😊
## Prerequisites

### Install Rust (with `rustup`)

If you don't have `rustup` installed yet, follow the instructions on the [rustup.rs site](https://rustup.rs)

### Install ESP32 toolchain 
If you haven't installed the ESP toolchain yet, here's the link to get started: [espressif](https://docs.espressif.com/projects/esp-idf/en/v5.0.2/esp32/get-started/index.html#manual-installation)

### Install Cargo Sub-Commands

```sh
cargo install cargo-generate
cargo install ldproxy
cargo install espup
cargo install espflash
```

## Let's get started

#### Copy source code
```sh
git clone https://github.com/lnB51/spark
```

#### Create a ```cfg.toml``` file in the root directory. You can use ```cfg.toml.example``` as a template — just update it with your own details.
```sh
[spark]
wifi_ssid = "Spark"
wifi_pass = "Remote"
bot_token = "Hello I'm bot token"
bot_owner_id = 12345678
```

P.S.: You can find your ```bot_owner_id``` by visiting [Web Telegram](https://web.telegram.org/a/). Just open your "Saved Messages" and check the webpage URL for the ID. To get your ```bot_token```, use [BotFather](https://telegram.me/BotFather) on Telegram.

If you're using VSCode, I recommend installing [Task runner](https://marketplace.visualstudio.com/items?itemName=SanaAjani.taskrunnercode). I've already set up a ```task.json``` file for you, so you can easily ```build, flash, and monitor``` with just one click!

Just a heads up—log monitoring from the board is only available in debug mode.

If you're not using VSCode, here are the commands you'll need:

#### Build (Debug)
```sh
cargo build
```

#### Build (Release)
```sh
cargo build --release
```

#### Flash
```sh
espflash flash target/riscv32imc-esp-espidf/debug/spark --list-all-ports
```

#### Monitor
```sh
espflash monitor
```

## Next steps

After you have uploaded the firmware to the board, you will need to solder the following [Circuit](https://github.com/lnB51/spark/blob/master/images/soldering_scheme.png)

To learn more you can use pinout diagram [Pinout](https://github.com/lnB51/spark/blob/master/images/pinout_scheme.jpg)

## Result

Everything is ready, now you can check your work by sending the following commands to the bot 🤖:

```/poweron``` - Turn on the computer

```/poweroff``` - Turn off the computer in normal mode

```/reboot``` - Force reboot the computer (use only if the computer is frozen or you cannot turn it off in the usual way)

## Modify the Code for Other Platforms

If you want to use a different board, check out the instructions here:

* [Rust on ESP-IDF](https://github.com/esp-rs/esp-idf-template/blob/master/README.md)
