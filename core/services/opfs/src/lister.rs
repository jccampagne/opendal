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

use wasm_bindgen_futures::JsFuture;

use super::error::*;
use opendal_core::raw::oio;
use opendal_core::*;

pub struct OpfsLister {
    /// The directory path being listed (e.g. "test-list/").
    path: String,
    iterator: js_sys::AsyncIterator,
}

// Safety: wasm32 is single-threaded, so Send/Sync are safe.
unsafe impl Send for OpfsLister {}
unsafe impl Sync for OpfsLister {}

impl OpfsLister {
    pub fn new(path: &str, iterator: js_sys::AsyncIterator) -> Self {
        Self {
            path: path.to_string(),
            iterator,
        }
    }
}

impl oio::List for OpfsLister {
    async fn next(&mut self) -> Result<Option<oio::Entry>> {
        let promise = self.iterator.next().map_err(parse_js_error)?;
        let result = JsFuture::from(promise).await.map_err(parse_js_error)?;

        let done = js_sys::Reflect::get(&result, &"done".into())
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if done {
            return Ok(None);
        }

        let value = js_sys::Reflect::get(&result, &"value".into()).map_err(parse_js_error)?;
        let arr = js_sys::Array::from(&value);
        let name = arr.get(0).as_string().unwrap_or_default();
        let handle = arr.get(1);

        let kind = js_sys::Reflect::get(&handle, &"kind".into())
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();

        let entry = if kind == "directory" {
            oio::Entry::new(
                &format!("{}{}/", self.path, name),
                Metadata::new(EntryMode::DIR),
            )
        } else {
            oio::Entry::new(
                &format!("{}{}", self.path, name),
                Metadata::new(EntryMode::FILE),
            )
        };

        Ok(Some(entry))
    }
}
