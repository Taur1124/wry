use crate::http::{
  header::{HeaderName, HeaderValue},
  RequestBuilder,
};
use tao::{
  platform::android::ndk_glue::jni::{
    errors::Error as JniError,
    objects::{JClass, JMap, JObject, JString},
    sys::jobject,
    JNIEnv,
  },
  window::Window,
};

use super::{MainPipe, WebViewMessage, IPC, REQUEST_HANDLER};

#[allow(non_snake_case)]
pub unsafe fn runInitializationScripts(_: JNIEnv, _: JClass, _: JObject) {
  MainPipe::send(WebViewMessage::RunInitializationScripts);
}

fn handle_request(env: JNIEnv, request: JObject) -> Result<jobject, JniError> {
  let mut request_builder = RequestBuilder::new();

  let uri = env
    .call_method(request, "getUrl", "()Landroid/net/Uri;", &[])?
    .l()?;
  let url: JString = env
    .call_method(uri, "toString", "()Ljava/lang/String;", &[])?
    .l()?
    .into();
  request_builder = request_builder.uri(&env.get_string(url)?.to_string_lossy().to_string());

  let method: JString = env
    .call_method(request, "getMethod", "()Ljava/lang/String;", &[])?
    .l()?
    .into();
  request_builder = request_builder.method(
    env
      .get_string(method)?
      .to_string_lossy()
      .to_string()
      .as_str(),
  );

  let request_headers = env
    .call_method(request, "getRequestHeaders", "()Ljava/util/Map;", &[])?
    .l()?;
  let request_headers = JMap::from_env(&env, request_headers)?;
  for (header, value) in request_headers.iter()? {
    let header = env.get_string(header.into())?;
    let value = env.get_string(value.into())?;
    if let (Ok(header), Ok(value)) = (
      HeaderName::from_bytes(header.to_bytes()),
      HeaderValue::from_bytes(value.to_bytes()),
    ) {
      request_builder = request_builder.header(header, value);
    }
  }

  if let Some(handler) = REQUEST_HANDLER.get() {
    let response = (handler.0)(request_builder.body(Vec::new()).unwrap());
    if let Some(response) = response {
      let status_code = response.status().as_u16() as i32;
      let reason_phrase = "OK";
      let encoding = "UTF-8";
      let mime_type = if let Some(mime) = response.mimetype() {
        env.new_string(mime)?.into()
      } else {
        JObject::null()
      };

      let hashmap = env.find_class("java/util/HashMap")?;
      let response_headers = env.new_object(hashmap, "()V", &[])?;
      for (key, value) in response.headers().iter() {
        env.call_method(
          response_headers,
          "put",
          "(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;",
          &[
            env.new_string(key.as_str())?.into(),
            // TODO can we handle this better?
            env
              .new_string(String::from_utf8_lossy(value.as_bytes()))?
              .into(),
          ],
        )?;
      }

      let bytes = response.body;

      let byte_array_input_stream = env.find_class("java/io/ByteArrayInputStream")?;
      let byte_array = env.byte_array_from_slice(&bytes)?;
      let stream = env.new_object(byte_array_input_stream, "([B)V", &[byte_array.into()])?;

      let web_resource_response_class = env.find_class("android/webkit/WebResourceResponse")?;
      let web_resource_response = env.new_object(
        web_resource_response_class,
        "(Ljava/lang/String;Ljava/lang/String;ILjava/lang/String;Ljava/util/Map;Ljava/io/InputStream;)V",
        &[mime_type.into(), env.new_string(encoding)?.into(), status_code.into(), env.new_string(reason_phrase)?.into(), response_headers.into(), stream.into()],
      )?;

      return Ok(*web_resource_response);
    }
  }
  Ok(*JObject::null())
}

#[allow(non_snake_case)]
pub unsafe fn handleRequest(env: JNIEnv, _: JClass, request: JObject) -> jobject {
  match handle_request(env, request) {
    Ok(response) => response,
    Err(e) => {
      log::error!("Failed to handle request: {}", e);
      *JObject::null()
    }
  }
}

pub unsafe fn ipc(env: JNIEnv, _: JClass, arg: JString) {
  match env.get_string(arg) {
    Ok(arg) => {
      let arg = arg.to_string_lossy().to_string();
      if let Some(w) = IPC.get() {
        let ipc = w.0;
        if !ipc.is_null() {
          let ipc = &*(ipc as *mut Box<dyn Fn(&Window, String)>);
          ipc(&w.1, arg)
        }
      }
    }
    Err(e) => log::error!("Failed to parse JString: {}", e),
  }
}
