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
use wasm_bindgen_futures::JsFuture;
use web_sys::FileSystemWritableFileStream;

use opendal_core::raw::oio;
use opendal_core::*;

use super::error::*;
use super::utils::*;

pub struct OpfsWriter {
    path: String,
}

impl OpfsWriter {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl oio::OneShotWrite for OpfsWriter {
    async fn write_once(&self, bs: Buffer) -> Result<Metadata> {
        let handle = get_file_handle(&self.path, true).await?;

        let writable: FileSystemWritableFileStream = JsFuture::from(handle.create_writable())
            .await
            .and_then(JsCast::dyn_into)
            .map_err(parse_js_error)?;

        let bytes = bs.to_bytes();

        // Copy data into a JS-owned Uint8Array. Using write_with_u8_array
        // directly passes a view into WASM linear memory, which Safari
        // can invalidate during the async write, producing corrupted data.
        let js_buf = js_sys::Uint8Array::new_with_length(bytes.len() as u32);
        js_buf.copy_from(&bytes);

        JsFuture::from(
            writable
                .write_with_buffer_source(&js_buf)
                .map_err(parse_js_error)?,
        )
        .await
        .map_err(parse_js_error)?;

        JsFuture::from(writable.close())
            .await
            .map_err(parse_js_error)?;

        let mut metadata = Metadata::new(EntryMode::FILE);
        metadata.set_content_length(bytes.len() as u64);

        Ok(metadata)
    }
}
