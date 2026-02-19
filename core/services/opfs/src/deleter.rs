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
use web_sys::FileSystemGetDirectoryOptions;
use web_sys::FileSystemRemoveOptions;

use opendal_core::raw::oio;
use opendal_core::raw::*;
use opendal_core::*;

use super::error::*;
use super::utils::*;

pub struct OpfsDeleter {
    root: String,
}

impl OpfsDeleter {
    pub fn new(root: String) -> Self {
        Self { root }
    }
}

impl oio::OneShotDelete for OpfsDeleter {
    async fn delete_once(&self, path: String, _args: OpDelete) -> Result<()> {
        let p = build_abs_path(&self.root, &path);
        let path = p.trim_matches('/');
        let parts: Vec<&str> = path.split('/').collect();

        // Get the parent directory handle.
        let parent = if parts.len() > 1 {
            let dir_opt = FileSystemGetDirectoryOptions::new();
            dir_opt.set_create(false);
            let parent_path = parts[..parts.len() - 1].join("/");
            get_directory_handle(&format!("{}/", parent_path), &dir_opt).await?
        } else {
            get_root_directory_handle().await?
        };

        let name = parts.last().expect("path must not be empty");

        let opts = FileSystemRemoveOptions::new();
        opts.set_recursive(true);

        // Ignore NotFound errors — delete should be idempotent.
        match JsFuture::from(parent.remove_entry_with_options(name, &opts)).await {
            Ok(_) => Ok(()),
            Err(e) => {
                let err_str = format!("{:?}", e);
                if err_str.contains("NotFoundError") {
                    Ok(())
                } else {
                    Err(parse_js_error(e))
                }
            }
        }
    }
}
