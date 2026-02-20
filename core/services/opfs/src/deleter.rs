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
use web_sys::FileSystemRemoveOptions;

use super::error::*;
use super::utils::*;
use opendal_core::raw::oio;
use opendal_core::raw::*;
use opendal_core::*;

pub struct OpfsDeleter {
    pub(super) root: String,
}

impl oio::OneShotDelete for OpfsDeleter {
    async fn delete_once(&self, path: String, _args: OpDelete) -> Result<()> {
        let rooted = build_rooted_abs_path(&self.root, &path);
        let (parent_handle, name) = match get_parent_handle(&rooted).await {
            Ok(v) => v,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e),
        };

        let opts = FileSystemRemoveOptions::new();
        opts.set_recursive(true);

        let result =
            JsFuture::from(parent_handle.remove_entry_with_options(&name, &opts)).await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                let err = parse_js_error(e);
                if err.kind() == ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(err)
                }
            }
        }
    }
}
