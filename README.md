# cargo-v5
A cargo subcommand for uploading programs to the Vex V5 robot brain.


This is designed for use with this project template: [hvex-v5-template](https://github.com/Culpeper-Robotics/vex-v5-template).
This is only confirmed to work on ubuntu linux.
Windows users can use wsl, however they will need to specify what port to use to connect to the v5. The aforementioned template automatically does it.


## Installation

```bash
rustup install nightly
rustup component add rust-src --toolchain nightly
sudo apt install libudev-dev libclang-dev
export LIBV5_PATH="/path/to/libv5rt/sdk/"
cargo install --git https://github.com/Culpeper-Robotics/cargo-v5
```