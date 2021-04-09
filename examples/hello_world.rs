// SPDX-License-Identifier: Apache-2.0 OR MIT

use wry::{Application, Attributes, Result};

fn main() -> Result<()> {
  let mut app = Application::new()?;

  let attributes = Attributes {
    url: Some("https://tauri.studio/".to_string()),
    title: String::from("Hello World!"),
    ..Default::default()
  };

  app.add_window(attributes)?;
  app.run();
  Ok(())
}
