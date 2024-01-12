<img src=".github/splash.png" alt="WRY Webview Rendering library" />

[![](https://img.shields.io/crates/v/wry?style=flat-square)](https://crates.io/crates/wry) [![](https://img.shields.io/docsrs/wry?style=flat-square)](https://docs.rs/wry/)
[![License](https://img.shields.io/badge/License-MIT%20or%20Apache%202-green.svg)](https://opencollective.com/tauri)
[![Chat Server](https://img.shields.io/badge/chat-discord-7289da.svg)](https://discord.gg/SpmNs4S)
[![website](https://img.shields.io/badge/website-tauri.app-purple.svg)](https://tauri.app)
[![https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg](https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg)](https://good-labs.github.io/greater-good-affirmation)
[![support](https://img.shields.io/badge/sponsor-Open%20Collective-blue.svg)](https://opencollective.com/tauri)

## Overview

This is the special branch of wry to experiment Servo, a web engine written mostly in Rust, as a crate dependency.
The motivation of this experiment is evaluating custom web egines that can be fully under our control and be customized at will.
And at the same time, finding the root cause and pivot point that could really improve and help web and rust community moving forward.
Servo fits into this position pretty well because it isn't controlled by any huge corporation. Evryone from the open source community is free to shape the project together.
While it doesn't provide full coverage of all web features yet, it already offers super flexible interface to work with.
In this branch, we showcase how to integrate and customize it to become a modern style landing page.

![](demo.png)
[Video link](https://twitter.com/Yu_Wei_Wu/status/1740251457285431487) to see the demo showcase

## Usage

The current demo works best on macOS at the moment, since it tries to customize its traffic light buttons to be seamless in the window.

It should also work on Windows, as well as Linux with X11. You may encounter problems running the demo on Linux with Wayland or Xwayland.

### Build Servo

- Clone Servo repository (rev@ 90f70e3): We are still working on making it to be a cargo git dependency. But it's more stable to make a local build for now.

  ```sh
  git clone https://github.com/servo/servo.git
  cd servo
  git checkout 90f70e3408e1d4b3f378e50f9f051cb00c77c446
  ```

  - Please follow the instructions in [Servo - Build Setup (macOS)](https://github.com/servo/servo#macos) to build a successful copy first.

- Build wry

  - Clone wry repository

  ```sh
  git clone https://github.com/tauri-apps/wry.git
  cd wry
  ```

  - Copy required files from Servo repository

    - macOS, Linux:

    ```sh
    cp -a ../servo/resources .
    cp -f ../servo/Cargo.lock .
    ```

    - Windows:

    ```
    xcopy /i ..\servo\resources resources
    copy ..\servo\Cargo.lock .
    ```

  - **Windows only:** set environment variables

  ```
  set PYTHON3=python
  set LIBCLANG_PATH=C:\Program Files\LLVM\lib
  set MOZTOOLS_PATH=%CD%\..\servo\target\dependencies\moztools\4.0
  set CC=clang-cl.exe
  set CXX=clang-cl.exe
  ```

  - **NixOS only:** add `wayland` and `libGL` to `LD_LIBRARY_PATH` in `../servo/etc/shell.nix`

  - Run servo example

  ```sh
  cargo run --example servo
  ```

    - Or if you are using Nix or NixOS:

    ```
    nix-shell ../servo/etc/shell.nix --run 'cargo run --example servo'
    ```

## Future Work

- Add more window and servo features to make it feel more like a general webview library.
- Improve Servo's development experience.
- Multi webviews and multi browsing contexts in the same window.