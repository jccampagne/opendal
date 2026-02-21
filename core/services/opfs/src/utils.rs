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

use opendal_core::Result;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::FileSystemDirectoryHandle;
use web_sys::FileSystemFileHandle;
use web_sys::FileSystemGetDirectoryOptions;
use web_sys::FileSystemGetFileOptions;
use web_sys::window;

use super::error::*;

/// Returns the root of OPFS
/// https://developer.mozilla.org/en-US/docs/Web/API/StorageManager/getDirectory
pub(crate) async fn get_root_directory_handle() -> Result<FileSystemDirectoryHandle> {
    let navigator = window().unwrap().navigator();
    let storage_manager = navigator.storage();
    JsFuture::from(storage_manager.get_directory())
        .await
        .and_then(JsCast::dyn_into)
        .map_err(parse_js_error)
}

/// Navigate to a directory by path, optionally creating intermediate directories.
pub(crate) async fn get_directory_handle(
    dir: &str,
    create: bool,
) -> Result<FileSystemDirectoryHandle> {
    let trimmed = dir.trim_matches('/');
    if trimmed.is_empty() {
        return get_root_directory_handle().await;
    }

    let opt = FileSystemGetDirectoryOptions::new();
    opt.set_create(create);

    let mut handle = get_root_directory_handle().await?;
    for segment in trimmed.split('/') {
        handle = JsFuture::from(handle.get_directory_handle_with_options(segment, &opt))
            .await
            .and_then(JsCast::dyn_into)
            .map_err(parse_js_error)?;
    }

    Ok(handle)
}

/// OPFS does not do subdirectory creation. We have to do it ourselves.
///
/// Split a path into (parent_dir, name).
/// "a/b/c.txt" -> ("a/b", "c.txt")
/// "file.txt" -> ("", "file.txt")
pub(crate) fn split_path(path: &str) -> (&str, &str) {
    let path = path.trim_matches('/');
    match path.rsplit_once('/') {
        Some((parent, name)) => (parent, name),
        None => ("", path),
    }
}

/// Get a file handle at an arbitrary path, optionally creating parent dirs and the file.
/// `path` must be a file, it cannot be a directory.
pub(crate) async fn get_file_handle(path: &str, create: bool) -> Result<FileSystemFileHandle> {
    debug_assert!(!path.ends_with('/'), "cannot be a directory");
    debug_assert!(path != "/", "cannot be root");

    let (parent, name) = split_path(path);

    let dir_handle = if parent.is_empty() {
        get_root_directory_handle().await?
    } else {
        get_directory_handle(parent, create).await?
    };

    let opt = FileSystemGetFileOptions::new();
    opt.set_create(create);

    JsFuture::from(dir_handle.get_file_handle_with_options(name, &opt))
        .await
        .and_then(JsCast::dyn_into)
        .map_err(parse_js_error)
}

/// Get the parent directory handle for a given path.
pub(crate) async fn get_parent_handle(path: &str) -> Result<(FileSystemDirectoryHandle, String)> {
    let (parent, name) = split_path(path);

    let dir_handle = if parent.is_empty() {
        get_root_directory_handle().await?
    } else {
        get_directory_handle(parent, false).await?
    };

    Ok((dir_handle, name.to_string()))
}
