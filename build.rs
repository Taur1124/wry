#[cfg(target_os = "linux")]
fn main() {}

#[cfg(target_os = "macos")]
fn main() {
    println!("cargo:rustc-link-lib=framework=WebKit");
}

#[cfg(target_os = "windows")]
fn main() {
    windows::build!(
        windows::foundation::collections::*
        windows::foundation::{AsyncStatus, Rect, Uri}
        windows::web::ui::{IWebViewControl, WebViewControlScriptNotifyEventArgs }
        windows::web::ui::interop::{WebViewControl, WebViewControlProcess }
        windows::win32::com::CoWaitForMultipleHandles
        windows::win32::display_devices::RECT
        windows::win32::system_services::{CreateEventA, SetEvent, INFINITE,GWL_EXSTYLE,WS_EX_TOOLWINDOW}
        windows::win32::windows_and_messaging::{GetClientRect, HWND, SetWindowLongA}
        windows::win32::winrt::RoInitialize
    );

    let mut build = cc::Build::new();
    build
        .include("src/collections.h")
        .file("src/collections.cpp")
        .flag_if_supported("/std:c++17");

    println!("cargo:rerun-if-changed=src/collections.h");
    println!("cargo:rerun-if-changed=src/collections.cpp");
    build.compile("collections");
}
