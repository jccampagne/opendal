// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use opendal_core::Error;
use opendal_core::ErrorKind;

pub fn console_fmt(f: &str, s: &impl std::fmt::Debug) {
    web_sys::console::log_1(&format!("{f}: {s:?}").into());
}

pub(crate) fn parse_js_error(value: JsValue) -> Error {
    let (name, message) = if let Some(js_err) = value.dyn_ref::<js_sys::Error>() {
        (js_err.name().into(), js_err.message().into())
    } else {
        let name = js_sys::Reflect::get(&value, &"name".into())
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| "UnknownError".to_string());

        let message = value
            .as_string()
            .or_else(|| {
                js_sys::Reflect::get(&value, &"message".into())
                    .ok()?
                    .as_string()
            })
            .unwrap_or_else(|| format!("{:?}", value));

        (name, message)
    };

    let kind = match name.as_str() {
        "NotFoundError" => ErrorKind::NotFound,
        "NotAllowedError" => ErrorKind::PermissionDenied,
        "TypeMismatchError" => ErrorKind::IsADirectory,
        "SecurityError" => ErrorKind::PermissionDenied,
        "QuotaExceededError" => ErrorKind::RateLimited, //? is that ok?
        // "NoModificationAllowedError" => ....
        _ => ErrorKind::Unexpected,
    };

    Error::new(kind, &message)
}
