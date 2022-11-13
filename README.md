# cargo-v5
![GitHub](https://img.shields.io/github/license/vexrs-os/cargo-v5?style=flat-square)
![GitHub repo size](https://img.shields.io/github/repo-size/vexrs-os/cargo-v5?style=flat-square)
![Crates.io](https://img.shields.io/crates/v/cargo-v5?style=flat-square)
![Libraries.io dependency status for latest release](https://img.shields.io/librariesio/release/cargo/cargo-v5?style=flat-square)


Cargo-v5 is a cargo subcommand for interacting with the Vex V5 robot brain.
Vex Software and Hardware was *NOT* reverse engineered to create this project. The protocol specification was derived from [PROS](https://pros.cs.purdue.edu), another piece of open source software that is targeted towards using C and C++ for programming Vex robotics. If you would prefer to use those two languages or are looking for a more complete and less minimal programming system, I would recommend you use PROS.

Not affiliated with Innovation First Inc.


## Usage

In order to download a valid program to the V5 brain, you will need to a vex target:
```bash
# This is slightly modified from the target found here: https://gitlab.com/qvex/vex-rt/-/blob/master/armv7a-vex-eabi.json
wget https://gist.githubusercontent.com/wireboy5/5bb41fe7bc8a0469635e56a3076946bf/raw/6699b6bed011447c724a4589dc3acb1e2ce61585/armv7a-vex-eabi.json
```

You can also add these contents to your cargo config file for convienience:
```toml
[build]
target = "build/armv7a-vex-eabi.json"

[unstable]
build-std-features = ["compiler-builtins-mem", "compiler-builtins-mangled-names"]
build-std = ["core", "alloc", "compiler_builtins"]

[target.armv7a-vex-eabi]
runner = "cargo v5 cargo-hook"

```

Now if you run `cargo run` it will compile and run your project on the v5 brain.

