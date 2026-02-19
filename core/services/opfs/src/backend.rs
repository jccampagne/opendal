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

use std::fmt::Debug;
use std::sync::Arc;

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::File;
use web_sys::FileSystemGetDirectoryOptions;

use super::OPFS_SCHEME;
use super::config::OpfsConfig;
use super::deleter::OpfsDeleter;
use super::error::*;
use super::lister::OpfsLister;
use super::utils::*;
use super::writer::OpfsWriter;
use opendal_core::raw::oio;
use opendal_core::raw::*;
use opendal_core::*;

#[doc = include_str!("docs.md")]
#[derive(Default, Debug)]
pub struct OpfsBuilder {
    pub(super) config: OpfsConfig,
}

impl Builder for OpfsBuilder {
    type Config = OpfsConfig;

    fn build(self) -> Result<impl Access> {
        let root = normalize_root(&self.config.root.unwrap_or_default());
        Ok(OpfsBackend { root })
    }
}

impl OpfsBuilder {
    /// Set root of this backend.
    ///
    /// All operations will happen under this root.
    pub fn root(mut self, root: &str) -> Self {
        self.config.root = if root.is_empty() {
            None
        } else {
            Some(root.to_string())
        };

        self
    }
}

/// OPFS Service backend
#[derive(Default, Debug, Clone)]
pub struct OpfsBackend {
    root: String,
}

impl Access for OpfsBackend {
    type Reader = Buffer;
    type Writer = oio::OneShotWriter<OpfsWriter>;
    type Lister = OpfsLister;
    type Deleter = oio::OneShotDeleter<OpfsDeleter>;

    fn info(&self) -> Arc<AccessorInfo> {
        let info = AccessorInfo::default();
        info.set_scheme(OPFS_SCHEME);
        info.set_name("opfs");
        info.set_root(&self.root);
        info.set_native_capability(Capability {
            create_dir: true,
            delete: true,
            list_with_recursive: true,
            list: true,
            read: true,
            stat: true,
            write: true,
            ..Default::default()
        });
        Arc::new(info)
    }

    async fn stat(&self, path: &str, _args: OpStat) -> Result<RpStat> {
        let p = build_abs_path(&self.root, path);

        if path.ends_with('/') {
            // Directory: just check it exists.
            let dir_opt = FileSystemGetDirectoryOptions::new();
            dir_opt.set_create(false);
            get_directory_handle(&p, &dir_opt).await?;

            let metadata = Metadata::new(EntryMode::DIR);
            return Ok(RpStat::new(metadata));
        }

        // File: get size and last_modified.
        let handle = get_file_handle(&p, false).await?;

        let file: File = JsFuture::from(handle.get_file())
            .await
            .and_then(JsCast::dyn_into)
            .map_err(parse_js_error)?;

        let mut metadata = Metadata::new(EntryMode::FILE);
        metadata.set_content_length(file.size() as u64);

        if let Ok(t) = Timestamp::from_millisecond(file.last_modified() as i64) {
            metadata.set_last_modified(t);
        }

        Ok(RpStat::new(metadata))
    }

    async fn read(&self, path: &str, args: OpRead) -> Result<(RpRead, Self::Reader)> {
        let p = build_abs_path(&self.root, path);
        let handle = get_file_handle(&p, false).await?;

        let file: File = JsFuture::from(handle.get_file())
            .await
            .and_then(JsCast::dyn_into)
            .map_err(parse_js_error)?;

        let array_buffer = JsFuture::from(file.array_buffer())
            .await
            .map_err(parse_js_error)?;

        let content = js_sys::Uint8Array::new(&array_buffer).to_vec();
        let buf = Buffer::from(content);
        let buf = buf.slice(args.range().to_range_as_usize());

        Ok((RpRead::default(), buf))
    }

    async fn write(&self, path: &str, _args: OpWrite) -> Result<(RpWrite, Self::Writer)> {
        let p = build_abs_path(&self.root, path);
        let writer = OpfsWriter::new(p);
        Ok((RpWrite::default(), oio::OneShotWriter::new(writer)))
    }

    async fn delete(&self) -> Result<(RpDelete, Self::Deleter)> {
        Ok((
            RpDelete::default(),
            oio::OneShotDeleter::new(OpfsDeleter::new(self.root.clone())),
        ))
    }

    async fn list(&self, path: &str, args: OpList) -> Result<(RpList, Self::Lister)> {
        let p = build_abs_path(&self.root, path);
        let dir_opt = FileSystemGetDirectoryOptions::new();
        dir_opt.set_create(false);
        let dir_handle = if p == "/" {
            get_root_directory_handle().await?
        } else {
            get_directory_handle(&p, &dir_opt).await?
        };

        let mut entries = Vec::new();
        collect_entries(&mut entries, &dir_handle, path, args.recursive()).await?;

        Ok((RpList::default(), OpfsLister::new(entries)))
    }

    async fn create_dir(&self, path: &str, _: OpCreateDir) -> Result<RpCreateDir> {
        let p = build_abs_path(&self.root, path);
        let opt = FileSystemGetDirectoryOptions::new();
        opt.set_create(true);
        get_directory_handle(&p, &opt).await?;

        Ok(RpCreateDir::default())
    }
}
