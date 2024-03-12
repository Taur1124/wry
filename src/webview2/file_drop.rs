// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// A silly implementation of file drop handling for Windows!

use crate::DragDropEvent;

use std::{
  cell::UnsafeCell,
  ffi::OsString,
  os::{raw::c_void, windows::ffi::OsStringExt},
  path::PathBuf,
  ptr,
  rc::Rc,
};

use windows::Win32::{
  Foundation::{self as win32f, BOOL, DRAGDROP_E_INVALIDHWND, HWND, LPARAM, POINT, POINTL},
  Graphics::Gdi::ScreenToClient,
  System::{
    Com::{IDataObject, DVASPECT_CONTENT, FORMATETC, TYMED_HGLOBAL},
    Ole::{
      IDropTarget, IDropTarget_Impl, RegisterDragDrop, RevokeDragDrop, CF_HDROP, DROPEFFECT,
      DROPEFFECT_COPY, DROPEFFECT_NONE,
    },
    SystemServices::MODIFIERKEYS_FLAGS,
  },
  UI::{
    Shell::{DragFinish, DragQueryFileW, HDROP},
    WindowsAndMessaging::EnumChildWindows,
  },
};

use windows_implement::implement;

#[derive(Default)]
pub(crate) struct FileDropController {
  drop_targets: Vec<IDropTarget>,
}

impl FileDropController {
  #[inline]
  pub(crate) fn new(hwnd: HWND, handler: Box<dyn Fn(DragDropEvent) -> bool>) -> Self {
    let mut controller = FileDropController::default();

    let handler = Rc::new(handler);

    // Enumerate child windows to find the WebView2 "window" and override!
    {
      let mut callback = |hwnd| controller.inject_in_hwnd(hwnd, handler.clone());
      let mut trait_obj: &mut dyn FnMut(HWND) -> bool = &mut callback;
      let closure_pointer_pointer: *mut c_void = unsafe { std::mem::transmute(&mut trait_obj) };
      let lparam = LPARAM(closure_pointer_pointer as _);
      unsafe extern "system" fn enumerate_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let closure = &mut *(lparam.0 as *mut c_void as *mut &mut dyn FnMut(HWND) -> bool);
        closure(hwnd).into()
      }
      unsafe { EnumChildWindows(hwnd, Some(enumerate_callback), lparam) };
    }

    controller
  }

  #[inline]
  fn inject_in_hwnd(&mut self, hwnd: HWND, handler: Rc<dyn Fn(DragDropEvent) -> bool>) -> bool {
    let file_drop_handler: IDropTarget = FileDropHandler::new(hwnd, handler).into();
    if unsafe { RevokeDragDrop(hwnd) } != Err(DRAGDROP_E_INVALIDHWND.into())
      && unsafe { RegisterDragDrop(hwnd, &file_drop_handler) }.is_ok()
    {
      self.drop_targets.push(file_drop_handler);
    }

    true
  }
}

#[implement(IDropTarget)]
pub struct FileDropHandler {
  hwnd: HWND,
  listener: Rc<dyn Fn(DragDropEvent) -> bool>,
  cursor_effect: UnsafeCell<DROPEFFECT>,
  enter_is_valid: UnsafeCell<bool>, /* If the currently hovered item is not valid there must not be any `HoveredFileCancelled` emitted */
}

impl FileDropHandler {
  pub fn new(hwnd: HWND, listener: Rc<dyn Fn(DragDropEvent) -> bool>) -> FileDropHandler {
    Self {
      hwnd,
      listener,
      cursor_effect: DROPEFFECT_NONE.into(),
      enter_is_valid: false.into(),
    }
  }

  unsafe fn iterate_filenames<F>(data_obj: Option<&IDataObject>, mut callback: F) -> Option<HDROP>
  where
    F: FnMut(PathBuf),
  {
    let drop_format = FORMATETC {
      cfFormat: CF_HDROP.0,
      ptd: ptr::null_mut(),
      dwAspect: DVASPECT_CONTENT.0,
      lindex: -1,
      tymed: TYMED_HGLOBAL.0 as u32,
    };

    match data_obj
      .as_ref()
      .expect("Received null IDataObject")
      .GetData(&drop_format)
    {
      Ok(medium) => {
        let hdrop = HDROP(medium.u.hGlobal.0 as _);

        // The second parameter (0xFFFFFFFF) instructs the function to return the item count
        let item_count = DragQueryFileW(hdrop, 0xFFFFFFFF, None);

        for i in 0..item_count {
          // Get the length of the path string NOT including the terminating null character.
          // Previously, this was using a fixed size array of MAX_PATH length, but the
          // Windows API allows longer paths under certain circumstances.
          let character_count = DragQueryFileW(hdrop, i, None) as usize;
          let str_len = character_count + 1;

          // Fill path_buf with the null-terminated file name
          let mut path_buf = Vec::with_capacity(str_len);
          DragQueryFileW(hdrop, i, std::mem::transmute(path_buf.spare_capacity_mut()));
          path_buf.set_len(str_len);

          callback(OsString::from_wide(&path_buf[0..character_count]).into());
        }

        Some(hdrop)
      }
      Err(error) => {
        log::warn!(
          "{}",
          match error.code() {
            win32f::DV_E_FORMATETC => {
              // If the dropped item is not a file this error will occur.
              // In this case it is OK to return without taking further action.
              "Error occurred while processing dropped/hovered item: item is not a file."
            }
            _ => "Unexpected error occurred while processing dropped/hovered item.",
          }
        );
        None
      }
    }
  }
}

#[allow(non_snake_case)]
impl IDropTarget_Impl for FileDropHandler {
  fn DragEnter(
    &self,
    pDataObj: Option<&IDataObject>,
    _grfKeyState: MODIFIERKEYS_FLAGS,
    pt: &POINTL,
    pdwEffect: *mut DROPEFFECT,
  ) -> windows::core::Result<()> {
    let mut pt = POINT { x: pt.x, y: pt.y };
    unsafe { ScreenToClient(self.hwnd, &mut pt) };

    let mut paths = Vec::new();
    let hdrop = unsafe { Self::iterate_filenames(pDataObj, |path| paths.push(path)) };
    (self.listener)(DragDropEvent::Enter {
      paths,
      position: (pt.x as _, pt.y as _),
    });

    unsafe {
      let enter_is_valid = hdrop.is_some();
      *self.enter_is_valid.get() = enter_is_valid;

      let cursor_effect = if enter_is_valid {
        DROPEFFECT_COPY
      } else {
        DROPEFFECT_NONE
      };
      *pdwEffect = cursor_effect;
      *self.cursor_effect.get() = cursor_effect;
    }

    Ok(())
  }

  fn DragOver(
    &self,
    _grfKeyState: MODIFIERKEYS_FLAGS,
    pt: &POINTL,
    pdwEffect: *mut DROPEFFECT,
  ) -> windows::core::Result<()> {
    if unsafe { *self.enter_is_valid.get() } {
      let mut pt = POINT { x: pt.x, y: pt.y };
      unsafe { ScreenToClient(self.hwnd, &mut pt) };
      (self.listener)(DragDropEvent::Over {
        position: (pt.x as _, pt.y as _),
      });
    }

    unsafe { *pdwEffect = *self.cursor_effect.get() };
    Ok(())
  }

  fn DragLeave(&self) -> windows::core::Result<()> {
    if unsafe { *self.enter_is_valid.get() } {
      (self.listener)(DragDropEvent::Leave);
    }
    Ok(())
  }

  fn Drop(
    &self,
    pDataObj: Option<&IDataObject>,
    _grfKeyState: MODIFIERKEYS_FLAGS,
    pt: &POINTL,
    _pdwEffect: *mut DROPEFFECT,
  ) -> windows::core::Result<()> {
    if unsafe { *self.enter_is_valid.get() } {
      let mut pt = POINT { x: pt.x, y: pt.y };
      unsafe { ScreenToClient(self.hwnd, &mut pt) };

      let mut paths = Vec::new();
      let hdrop = unsafe { Self::iterate_filenames(pDataObj, |path| paths.push(path)) };
      (self.listener)(DragDropEvent::Drop {
        paths,
        position: (pt.x as _, pt.y as _),
      });

      if let Some(hdrop) = hdrop {
        unsafe { DragFinish(hdrop) };
      }
    }

    Ok(())
  }
}
