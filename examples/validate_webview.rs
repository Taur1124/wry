// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

fn main() -> wry::Result<()> {
  use wry::{
    application::{
      event::{Event, StartCause, WindowEvent},
      event_loop::{ControlFlow, EventLoop},
      window::WindowBuilder,
    },
    webview::{webview_version, WebViewBuilder},
  };

  // make sure webview is available
  match webview_version() {
    Ok(current_version) => {
      println!(
        "Webview ({}) available, initializing wry...",
        current_version
      );

      let event_loop = EventLoop::new();
      let window = WindowBuilder::new()
        .with_title("Hello World")
        .build(&event_loop)?;
      let _webview = WebViewBuilder::new(window)?
        .with_url("https://tauri.studio")?
        .build()?;

      event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
          Event::NewEvents(StartCause::Init) => println!("Wry has started!"),
          Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
          } => *control_flow = ControlFlow::Exit,
          _ => (),
        }
      });
    }
    Err(error) => {
      println!("Unable to get webview version: {}", error);
    }
  };

  Ok(())
}
