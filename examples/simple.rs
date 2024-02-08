// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tao::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder,
};
use wry::WebViewBuilder;

fn main() -> wry::Result<()> {
  let mut event_loop = EventLoop::new();
  let window = WindowBuilder::new().build(&event_loop).unwrap();

  #[cfg(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "ios",
    target_os = "android"
  ))]
  let builder = WebViewBuilder::new(&window);

  #[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "ios",
    target_os = "android"
  )))]
  let builder = {
    use tao::platform::unix::WindowExtUnix;
    use wry::WebViewBuilderExtUnix;
    let vbox = window.default_vbox().unwrap();
    WebViewBuilder::new_gtk(vbox)
  };

  let _webview = builder.with_url("https://tauri.app")?.build()?;

  loop {
    use tao::platform::run_return::EventLoopExtRunReturn;
    event_loop.run_return(move |event, _, control_flow| {
      *control_flow = ControlFlow::Wait;

      if event == Event::MainEventsCleared {
        *control_flow = ControlFlow::Exit;
      }

      if let Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } = event
      {
        *control_flow = ControlFlow::Exit
      }
    });
  }
}
