<img src=".github/splash.png" alt="WRY Webview Rendering library" />

[![](https://img.shields.io/crates/v/wry?style=flat-square)](https://crates.io/crates/wry) [![](https://img.shields.io/docsrs/wry?style=flat-square)](https://docs.rs/wry/)
[![License](https://img.shields.io/badge/License-MIT%20or%20Apache%202-green.svg)](https://opencollective.com/tauri)
[![Chat Server](https://img.shields.io/badge/chat-discord-7289da.svg)](https://discord.gg/SpmNs4S)
[![website](https://img.shields.io/badge/website-tauri.app-purple.svg)](https://tauri.app)
[![https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg](https://good-labs.github.io/greater-good-affirmation/assets/images/badge.svg)](https://good-labs.github.io/greater-good-affirmation)
[![support](https://img.shields.io/badge/sponsor-Open%20Collective-blue.svg)](https://opencollective.com/tauri)

## Overview

This is a special branch of `wry` to experiment with Servo, a web engine written mostly in Rust and currently a Linux Foundation project, as a crate dependency.
The motivation for this experiment is evaluating custom web engines that can be fully controlled and customized at will.
And at the same time, finding the root cause and pivot point that could really improve and help the web and Rust communities moving forward.
Servo fits into this position pretty well because it isn't controlled by any huge corporation. Evryone from the open source community is free to shape the project together.
While Servo doesn't provide full coverage of all web features yet, it already offers a super-flexible interface to work with.
In this branch, we showcase how to integrate and customize Servo to power a modern style landing page within a Tauri window.

![](demo.png)
[Video link](https://twitter.com/Yu_Wei_Wu/status/1740251457285431487) to see the demo showcase

## Usage

The current demo works best on macOS at the moment, since it tries to customize it's traffic light buttons to be seamless in the window.

The demo should also work on Windows as well as X11 on Linux. You may encounter problems running the demo on Linux with Wayland or Xwayland.

### Build

- Please follow the instructions in [Servo - Build Setup (macOS)](https://github.com/servo/servo#macos) to build a successful copy first.

- Download [mozjs binary](https://github.com/wusyong/mozjs/releases/tag/tag-8410b587d66a36f1660cc3b828359e199eb0760a) and set the environment variable:

```sh
MOZJS_MIRROR=path/to/libmozjs.tar.gz
```

- Build wry

  - Clone wry repository

  ```sh
  git clone https://github.com/tauri-apps/wry.git
  cd wry
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

- Add more window and Servo features to make it feel more like a genuine WebView library
- Improve Servo's development experience
- Multi webviews and multi browsing contexts in the same window.
