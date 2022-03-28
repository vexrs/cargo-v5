# cargo-v5
![GitHub](https://img.shields.io/github/license/Culpeper-Robotics/cargo-v5?style=flat-square)
![GitHub repo size](https://img.shields.io/github/repo-size/Culpeper-Robotics/cargo-v5?style=flat-square)
![Crates.io](https://img.shields.io/crates/v/cargo-v5?style=flat-square)
![Libraries.io dependency status for latest release](https://img.shields.io/librariesio/release/cargo/cargo-v5?style=flat-square)


A lightweight cargo subcommand for uploading programs to the Vex V5 robot brain.
This is only known to work on Ubuntu Linux, although there should be no platform-specific code.


## Installation


### Ubuntu linux
```bash
rustup install nightly
rustup component add rust-src --toolchain nightly
sudo apt install libudev-dev libclang-dev
export LIBV5_PATH="/path/to/libv5rt/sdk/"
cargo install cargo-v5
```

## Usage

Create a rust crate:
```bash
mkdir project-name
cd project-name
cargo init
```

Download the vex toolchain file to your project's root directory:
```
wget https://gist.githubusercontent.com/wireboy5/5bb41fe7bc8a0469635e56a3076946bf/raw/6699b6bed011447c724a4589dc3acb1e2ce61585/armv7a-vex-eabi.json
```

Create a cargo config for your project:
```bash
mkdir .cargo
touch .cargo/config
```

Add these contents:
```toml
[build]
target = ".v5/armv7a-vex-eabi.json"

[unstable]
build-std = ["core", "alloc"]

[target.armv7a-vex-eabi]
runner = "cargo v5 upload --run"
```

Now if you run `cargo run` it will compile and run your project on the v5 brain.
These instructions can be adapted to work for windows.
You can use a library such as [vexv5rt](https://github.com/Culpeper-Robotics/vexv5rt) to interface with the v5 brain and devices.

## WSL Usage

This project works under an Ubuntu WSL2 installation. However, every time you plug in the v5 brain or controller you will need to forward the usb device to WSL.
Open Powershell as administrator, and run this command to find the vex device:
```powershell
usbipd wsl list
```
Once you have found the busid of your device (the two hyphenated numbers on the left of the device name) then you can attach the device to your WSL instance:
```powershell
usbipd wsl attach --busid {busid}
```
This last command can be run without checking the busid as long as you are not connecting and reconnecting other devices.