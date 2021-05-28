// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::convert::TryFrom;

use std::{path::PathBuf, rc::Rc};

use cairo::ImageSurface;
use gdk::{WindowEdge, WindowExt, RGBA};
use gio::Cancellable;
use glib::{signal::Inhibit, Bytes, Cast, FileError};
use gtk::{BoxExt, ContainerExt, WidgetExt};
use url::Url;
use webkit2gtk::{
  SecurityManagerExt, SettingsExt, SnapshotOptions, SnapshotRegion, URISchemeRequestExt,
  UserContentInjectedFrames, UserContentManager, UserContentManagerExt, UserScript,
  UserScriptInjectionTime, WebContextBuilder, WebContextExt, WebView, WebViewExt, WebViewExtManual,
  WebsiteDataManagerBuilder,
};
use webkit2gtk_sys::{
  webkit_get_major_version, webkit_get_micro_version, webkit_get_minor_version,
};

use crate::{
  application::{platform::unix::*, window::Window},
  webview::{mimetype::MimeType, FileDropEvent, RpcRequest, RpcResponse, ScreenshotRegion},
  Error, Result,
};

mod file_drop;

pub struct InnerWebView {
  webview: Rc<WebView>,
}

impl InnerWebView {
  pub fn new(
    window: Rc<Window>,
    scripts: Vec<String>,
    url: Option<Url>,
    transparent: bool,
    custom_protocols: Vec<(
      String,
      Box<dyn Fn(&Window, &str) -> Result<Vec<u8>> + 'static>,
    )>,
    rpc_handler: Option<Box<dyn Fn(&Window, RpcRequest) -> Option<RpcResponse>>>,
    file_drop_handler: Option<Box<dyn Fn(&Window, FileDropEvent) -> bool>>,
    data_directory: Option<PathBuf>,
  ) -> Result<Self> {
    let window_rc = Rc::clone(&window);
    let window = &window.gtk_window();
    // Webview widget
    let manager = UserContentManager::new();
    let mut context_builder = WebContextBuilder::new();
    if let Some(data_directory) = data_directory {
      let data_manager = WebsiteDataManagerBuilder::new()
        .local_storage_directory(
          &data_directory
            .join("localstorage")
            .to_string_lossy()
            .into_owned(),
        )
        .indexeddb_directory(
          &data_directory
            .join("databases")
            .join("indexeddb")
            .to_string_lossy()
            .into_owned(),
        )
        .build();
      context_builder = context_builder.website_data_manager(&data_manager);
    }
    let context = context_builder.build();

    let webview = Rc::new(WebView::new_with_context_and_user_content_manager(
      &context, &manager,
    ));

    // Message handler
    let wv = Rc::clone(&webview);
    let w = window_rc.clone();
    manager.register_script_message_handler("external");
    manager.connect_script_message_received(move |_m, msg| {
      if let (Some(js), Some(context)) = (msg.get_value(), msg.get_global_context()) {
        if let Some(js) = js.to_string(&context) {
          if let Some(rpc_handler) = rpc_handler.as_ref() {
            match super::rpc_proxy(&w, js, rpc_handler) {
              Ok(result) => {
                if let Some(ref script) = result {
                  let cancellable: Option<&Cancellable> = None;
                  wv.run_javascript(script, cancellable, |_| ());
                }
              }
              Err(e) => {
                eprintln!("{}", e);
              }
            }
          }
        }
      }
    });

    webview.connect_button_press_event(|webview, event| {
      if event.get_button() == 1 {
        let (cx, cy) = event.get_root();
        if let Some(window) = webview.get_parent_window() {
          let result = crate::application::platform::unix::hit_test(&window, cx, cy);

          // this check is necessary, otherwise the webview won't recieve the click properly when resize isn't needed
          if result != WindowEdge::__Unknown(8) {
            window.begin_resize_drag(result, 1, cx as i32, cy as i32, event.get_time());
          }
        }
      }
      Inhibit(false)
    });

    // Gtk application window can only contain one widget at a time.
    // In tao, we add a gtk box if menu bar is required. So we check if
    // there's a box widget here.
    if let Some(widget) = window.get_children().pop() {
      let vbox = widget.downcast::<gtk::Box>().unwrap();
      vbox.pack_start(&*webview, true, true, 0);
    } else {
      window.add(&*webview);
    }
    webview.grab_focus();

    // Enable webgl, webaudio, canvas features and others as default.
    if let Some(settings) = WebViewExt::get_settings(&*webview) {
      settings.set_enable_webgl(true);
      settings.set_enable_webaudio(true);
      settings.set_enable_accelerated_2d_canvas(true);
      settings.set_javascript_can_access_clipboard(true);

      // Enable App cache
      settings.set_enable_offline_web_application_cache(true);
      settings.set_enable_page_cache(true);

      debug_assert_eq!(
        {
          settings.set_enable_developer_extras(true);
        },
        ()
      );
    }

    // Transparent
    if transparent {
      webview.set_background_color(&RGBA {
        red: 0.,
        green: 0.,
        blue: 0.,
        alpha: 0.,
      });
    }

    // File drop handling
    if let Some(file_drop_handler) = file_drop_handler {
      file_drop::connect_drag_event(webview.clone(), window_rc.clone(), file_drop_handler);
    }

    if window.get_visible() {
      window.show_all();
    }

    let w = Self { webview };

    // Initialize scripts
    w.init("window.external={invoke:function(x){window.webkit.messageHandlers.external.postMessage(x);}}")?;
    for js in scripts {
      w.init(&js)?;
    }

    // Custom protocol
    for (name, handler) in custom_protocols {
      context
        .get_security_manager()
        .ok_or(Error::MissingManager)?
        .register_uri_scheme_as_secure(&name);
      let w = window_rc.clone();
      context.register_uri_scheme(&name.clone(), move |request| {
        if let Some(uri) = request.get_uri() {
          let uri = uri.as_str();

          match handler(&w, uri) {
            Ok(buffer) => {
              let mime = MimeType::parse(&buffer, uri);
              let input = gio::MemoryInputStream::from_bytes(&Bytes::from(&buffer));
              request.finish(&input, buffer.len() as i64, Some(&mime))
            }
            Err(_) => request.finish_error(&mut glib::Error::new(
              FileError::Exist,
              "Could not get requested file.",
            )),
          }
        } else {
          request.finish_error(&mut glib::Error::new(
            FileError::Exist,
            "Could not get uri.",
          ));
        }
      });
    }

    // Navigation
    if let Some(url) = url {
      w.webview.load_uri(url.as_str());
    }

    Ok(w)
  }

  pub fn print(&self) {
    let _ = self.eval("window.print()");
  }

  pub fn screenshot<F>(&self, region: ScreenshotRegion, handler: F) -> Result<()>
  where
    F: Fn(Result<Vec<u8>>) -> () + 'static + Send,
  {
    let cancellable: Option<&Cancellable> = None;
    let cb = move |result: std::result::Result<cairo::Surface, glib::Error>| match result {
      Ok(surface) => match ImageSurface::try_from(surface) {
        Ok(image) => {
          let mut bytes = Vec::new();
          match image.write_to_png(&mut bytes) {
            Ok(_) => handler(Ok(bytes)),
            Err(err) => handler(Err(Error::CairoIoError(err))),
          }
        }
        Err(_) => handler(Err(Error::CairoError(cairo::Error::SurfaceTypeMismatch))),
      },
      Err(err) => handler(Err(Error::GlibError(err))),
    };

    match region {
      ScreenshotRegion::FullDocument => {
        self.webview.get_snapshot(
          SnapshotRegion::FullDocument,
          SnapshotOptions::NONE,
          cancellable,
          cb,
        );
      }
      ScreenshotRegion::Visible => {
        self.webview.get_snapshot(
          SnapshotRegion::Visible,
          SnapshotOptions::NONE,
          cancellable,
          cb,
        );
      }
    };

    Ok(())
  }

  pub fn eval(&self, js: &str) -> Result<()> {
    let cancellable: Option<&Cancellable> = None;
    self.webview.run_javascript(js, cancellable, |_| ());
    Ok(())
  }

  fn init(&self, js: &str) -> Result<()> {
    if let Some(manager) = self.webview.get_user_content_manager() {
      let script = UserScript::new(
        js,
        UserContentInjectedFrames::TopFrame,
        UserScriptInjectionTime::Start,
        &[],
        &[],
      );
      manager.add_script(&script);
    } else {
      return Err(Error::InitScriptError);
    }
    Ok(())
  }
}

pub fn platform_webview_version() -> Result<String> {
  let (major, minor, patch) = unsafe {
    (
      webkit_get_major_version(),
      webkit_get_minor_version(),
      webkit_get_micro_version(),
    )
  };
  Ok(format!("{}.{}.{}", major, minor, patch).into())
}
