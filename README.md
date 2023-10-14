# OrbitingSandRust

A resource management and mining game simulating a realistic solar system economy.

# Platform Choices

## Reasons for Moving to Rust from C++

1. Multiprocessing - We need a A LOT of parallelism to make this work. Rust is a great language for this.
2. ECS - Without an engine, we need an ecs to impose some structure on our code. Rust has great ECS libraries.
3. No Cmake - Need I say more?

## Reaons for choosing GGEZ

* I tried godot C#, it didn't have good testing support and I felt it would be too slow or need too many modifications
* I tried raylib C++, and it didn't have support for a lot of mesh indices
* I tried rust macroquad, and it didn't have good mesh support
* Ggez has a lot of support for meshes, uses wgpu and other existing libraries, is 2D native and is based on LOVE. Really a match made in heaven.

## Challenges with Rust

1. We are not going to use Bevy because it is so alpha. Many of our libraries are alpha.
2. Bindings - We need to bind to liquidfun, which will be a pain.
3. Major refactors are a lot of work due to the borrow checker, but at least its always safe

# Installation

## Windows, Mac, Linux

Just download rust and `cargo run`

## WSL

It really doesn't work. Read the markdown comments if you want to attempt from my last stopping point.

<!--
There are a lot of dependencies:

```
sudo apt update

# The pkg-config command could not be found.
sudo apt install pkg-config

# could not find system library 'alsa' required by the 'alsa-sys' crate
sudo apt install libasound2-dev

# could not find system library 'libudev' required by the 'libudev-sys' crate
sudo apt install libusb-1.0-0-dev libftdi1-dev
sudo apt install libudev-dev

# We want to install perf
# REF: https://gist.github.com/abel0b/b1881e41b9e1c4b16d84e5e083c38a13
# windows
wsl --update
# wsl 2
sudo apt update
sudo apt install flex bison
sudo apt install libdwarf-dev libelf-dev libnuma-dev libunwind-dev \
libnewt-dev libdwarf++0 libelf++0 libdw-dev libbfb0-dev \
systemtap-sdt-dev libssl-dev libperl-dev python-dev-is-python3 \
binutils-dev libiberty-dev libzstd-dev libcap-dev libbabeltrace-dev
git clone https://github.com/microsoft/WSL2-Linux-Kernel --depth 1
cd WSL2-Linux-Kernel/tools/perf
make -j8 # parallel build
sudo cp perf /usr/local/bin
``` -->