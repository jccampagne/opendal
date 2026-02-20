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

use wasm_bindgen::JsValue;

use opendal_core::Error;
use opendal_core::ErrorKind;

pub fn console_debug(s: &impl std::fmt::Debug) {
    web_sys::console::log_1(&format!("value: {:?}", s).into())
    // web_sys::console::log_1(&format!("value: {:?}", res).into())
    // web_sys::console::log_1(.);
}

pub(crate) fn parse_js_error(value: JsValue) -> Error {
    let kind = js_sys::Reflect::get(&value, &"name".into())
        .ok()
        .and_then(|v| v.as_string())
        .map(|name| match name.as_str() {
            "NotFoundError" => ErrorKind::NotFound,
            "NotAllowedError" => ErrorKind::PermissionDenied,
            "TypeMismatchError" => ErrorKind::IsADirectory,
            _ => ErrorKind::Unexpected,
        })
        .unwrap_or(ErrorKind::Unexpected);

    let message = value
        .as_string()
        .or_else(|| {
            js_sys::Reflect::get(&value, &"message".into())
                .ok()
                .and_then(|v| v.as_string())
        })
        .unwrap_or_default();

    Error::new(kind, message)
}
