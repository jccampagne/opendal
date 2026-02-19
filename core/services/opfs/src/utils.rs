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

use opendal_core::raw::oio;
use opendal_core::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::FileSystemDirectoryHandle;
use web_sys::FileSystemFileHandle;
use web_sys::FileSystemGetDirectoryOptions;
use web_sys::FileSystemGetFileOptions;
use web_sys::FileSystemHandle;
use web_sys::window;

use super::error::*;

pub(crate) async fn get_root_directory_handle() -> Result<FileSystemDirectoryHandle> {
    let navigator = window().unwrap().navigator();
    let storage_manager = navigator.storage();
    JsFuture::from(storage_manager.get_directory())
        .await
        .and_then(JsCast::dyn_into)
        .map_err(parse_js_error)
}

pub(crate) async fn get_directory_handle(
    dir: &str,
    dir_opt: &FileSystemGetDirectoryOptions,
) -> Result<FileSystemDirectoryHandle> {
    let dirs: Vec<&str> = dir.trim_matches('/').split('/').collect();

    let mut handle = get_root_directory_handle().await?;
    for dir in dirs {
        handle = JsFuture::from(handle.get_directory_handle_with_options(dir, dir_opt))
            .await
            .and_then(JsCast::dyn_into)
            .map_err(parse_js_error)?;
    }

    Ok(handle)
}

pub(crate) async fn get_handle_by_filename(filename: &str) -> Result<FileSystemFileHandle> {
    get_file_handle(filename, true).await
}

pub(crate) async fn get_file_handle(path: &str, create: bool) -> Result<FileSystemFileHandle> {
    let path = path.trim_matches('/');
    let parts: Vec<&str> = path.split('/').collect();

    // web_sys::console::log_1(&format!("parts: {:?}", &parts).into());

    // Navigate to the parent directory, creating intermediate dirs if needed.
    let mut dir_handle = get_root_directory_handle().await?;
    if parts.len() > 1 {
        let dir_opt = FileSystemGetDirectoryOptions::new();
        dir_opt.set_create(create);
        for dir in &parts[..parts.len() - 1] {
            dir_handle =
                JsFuture::from(dir_handle.get_directory_handle_with_options(dir, &dir_opt))
                    .await
                    .and_then(JsCast::dyn_into)
                    .map_err(parse_js_error)?;
        }
    }

    let file_name = parts.last().expect("path must not be empty");
    let file_opt = FileSystemGetFileOptions::new();
    file_opt.set_create(create);

    JsFuture::from(dir_handle.get_file_handle_with_options(file_name, &file_opt))
        .await
        .and_then(JsCast::dyn_into)
        .map_err(parse_js_error)
}

/// Collect directory entries from a FileSystemDirectoryHandle.
///
/// Each iteration of `entries()` yields a `[name, handle]` JS array pair.
pub(crate) async fn collect_entries(
    out: &mut Vec<oio::Entry>,
    dir_handle: &FileSystemDirectoryHandle,
    prefix: &str,
    recursive: bool,
) -> Result<()> {
    let iter = dir_handle.entries();

    loop {
        let next = JsFuture::from(iter.next().map_err(parse_js_error)?)
            .await
            .map_err(parse_js_error)?;

        let done = js_sys::Reflect::get(&next, &"done".into())
            .map_err(parse_js_error)?
            .as_bool()
            .unwrap_or(true);

        if done {
            break;
        }

        let value = js_sys::Reflect::get(&next, &"value".into()).map_err(parse_js_error)?;
        let pair = js_sys::Array::from(&value);
        let name: String = pair.get(0).as_string().unwrap_or_default();
        let handle: FileSystemHandle = pair.get(1).dyn_into().map_err(parse_js_error)?;

        let kind = handle.kind();
        match kind {
            web_sys::FileSystemHandleKind::File => {
                let path = format!("{}{}", prefix, name);
                out.push(oio::Entry::new(&path, Metadata::new(EntryMode::FILE)));
            }
            web_sys::FileSystemHandleKind::Directory => {
                let path = format!("{}{}/", prefix, name);
                out.push(oio::Entry::new(&path, Metadata::new(EntryMode::DIR)));

                if recursive {
                    let sub_dir: FileSystemDirectoryHandle = handle.unchecked_into();
                    Box::pin(collect_entries(out, &sub_dir, &path, true)).await?;
                }
            }
            _ => {}
        }
    }

    Ok(())
}
