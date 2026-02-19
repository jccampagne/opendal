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

pub(crate) fn parse_js_error(val: JsValue) -> Error {
    let val = match val.dyn_into::<web_sys::DomException>() {
        Ok(ex) => {
            let kind = match ex.name().as_str() {
                "NotFoundError"  => ErrorKind::NotFound,
                // file instead of directory, vice-versa
                "TypeMismatchError" => ErrorKind::NotFound,
                "NotAllowedError" => ErrorKind::PermissionDenied,
                _ => ErrorKind::Unexpected,
            };
            return Error::new(kind, ex.message());
        }
        Err(val) => val,
    };

    let val = match val.dyn_into::<js_sys::Error>() {
        Ok(err) => return Error::new(ErrorKind::Unexpected, String::from(err.message())),
        Err(val) => val,
    };

    Error::new(ErrorKind::Unexpected, format!("{val:?}"))
}
