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
use web_sys::File;

use opendal_core::raw::*;
use opendal_core::*;

use super::error::*;
use super::utils::*;

pub async fn opfs_stat(path: &str) -> Result<Metadata> {
    if path == "/" || path.ends_with('/') {
        // For directories, just verify it exists by getting the handle.
        let trimmed = path.trim_matches('/');
        if trimmed.is_empty() {
            // Root always exists.
            return Ok(Metadata::new(EntryMode::DIR));
        }
        get_directory_handle(trimmed, false).await?;
        return Ok(Metadata::new(EntryMode::DIR));
    }

    let handle = get_file_handle(path, false).await?;
    let file: File = JsFuture::from(handle.get_file())
        .await
        .and_then(JsCast::dyn_into)
        .map_err(parse_js_error)?;

    let size = file.size() as u64;
    let last_modified_ms = file.last_modified() as i64;
    let mut meta = Metadata::new(EntryMode::FILE).with_content_length(size);
    if let Ok(ts) = Timestamp::from_millisecond(last_modified_ms) {
        meta = meta.with_last_modified(ts);
    }
    Ok(meta)
}

pub async fn opfs_read(path: &str) -> Result<Buffer> {
    let handle = get_file_handle(path, false).await?;
    let file: File = JsFuture::from(handle.get_file())
        .await
        .and_then(JsCast::dyn_into)
        .map_err(parse_js_error)?;
    let array_buffer = JsFuture::from(file.array_buffer())
        .await
        .map_err(parse_js_error)?;
    let bytes = js_sys::Uint8Array::new(&array_buffer).to_vec();
    Ok(Buffer::from(bytes))
}

pub async fn opfs_list(path: &str) -> Result<js_sys::AsyncIterator> {
    let trimmed = path.trim_matches('/');
    let dir_handle = if trimmed.is_empty() {
        get_root_directory_handle().await?
    } else {
        get_directory_handle(trimmed, false).await?
    };

    // Call entries() on the directory handle via JS reflection.
    let entries_fn: js_sys::Function =
        js_sys::Reflect::get(dir_handle.as_ref(), &"entries".into())
            .and_then(JsCast::dyn_into)
            .map_err(parse_js_error)?;

    entries_fn
        .call0(dir_handle.as_ref())
        .and_then(JsCast::dyn_into)
        .map_err(parse_js_error)
}
