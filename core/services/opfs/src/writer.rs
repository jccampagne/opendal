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

use js_sys::Object;
use js_sys::Reflect;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::File;
use web_sys::FileSystemWritableFileStream;
use web_sys::WriteParams;

use super::error::*;
use super::utils::*;
use opendal_core::raw::oio;
use opendal_core::raw::*;
use opendal_core::*;

pub struct OpfsWriter {
    path: String,
    buffer: Option<Vec<u8>>,
}

impl OpfsWriter {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            buffer: Some(Vec::new()),
        }
    }
}

impl oio::Write for OpfsWriter {
    async fn write(&mut self, bs: Buffer) -> Result<()> {
        console_fmt("write to pathx = {:?}", &self.path);
        console_fmt("bs = {:?}", &bs);
        let buf = self.buffer.get_or_insert_with(Vec::new);
        buf.extend_from_slice(&bs.to_vec());
        Ok(())
    }

    async fn close(&mut self) -> Result<Metadata> {
        let content = self.buffer.take().unwrap_or_default();

        let handle = get_file_handle(&self.path, true).await?;

        let writable: FileSystemWritableFileStream = JsFuture::from(handle.create_writable())
            .await
            .and_then(JsCast::dyn_into)
            .map_err(parse_js_error)?;

        let size = content.len() as u32;
        let params = WriteParams::new(web_sys::WriteCommandType::Write);
        params.set_size(Some(content.len() as f64));
        let js_value: &JsValue = &content.into();
        params.set_data(&js_value);
        console_fmt("writing with params = {}", &params);
        let promise = writable.write_with_write_params(&params.into());

        JsFuture::from(writable.close())
            .await
            .map_err(parse_js_error)?;

        let file: File = JsFuture::from(handle.get_file())
            .await
            .and_then(JsCast::dyn_into)
            .map_err(parse_js_error)?;

        let size = file.size() as u64;

        console_fmt("size = {:?}", &size);
        let last_modified_ms = file.last_modified() as i64;
        let mut meta = Metadata::new(EntryMode::FILE).with_content_length(size);
        if let Ok(ts) = Timestamp::from_millisecond(last_modified_ms) {
            meta = meta.with_last_modified(ts);
        }
        Ok(meta)
    }

    async fn abort(&mut self) -> Result<()> {
        self.buffer = None;
        Ok(())
    }
}
