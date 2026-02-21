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

use super::OPFS_SCHEME;
use super::config::OpfsConfig;
use super::core::*;
use super::deleter::OpfsDeleter;
use super::lister::OpfsLister;
use super::reader::OpfsReader;
use super::utils::*;
use super::writer::OpfsWriter;
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

/// OPFS Service backend
#[derive(Debug, Clone)]
pub struct OpfsBackend {
    root: String,
}

impl Access for OpfsBackend {
    type Reader = OpfsReader;
    type Writer = OpfsWriter;
    type Lister = Option<OpfsLister>;
    type Deleter = oio::OneShotDeleter<OpfsDeleter>;

    fn info(&self) -> Arc<AccessorInfo> {
        let info = AccessorInfo::default();
        info.set_scheme(OPFS_SCHEME);
        info.set_name("opfs");
        info.set_root(&self.root);
        info.set_native_capability(Capability {
            stat: true,
            read: true,
            write: true,
            write_can_empty: true,
            create_dir: true,
            delete: true,
            list: true,
            ..Default::default()
        });
        Arc::new(info)
    }

    async fn create_dir(&self, path: &str, _: OpCreateDir) -> Result<RpCreateDir> {
        let rooted = build_rooted_abs_path(&self.root, path);
        get_directory_handle(rooted.trim_matches('/'), true).await?;
        Ok(RpCreateDir::default())
    }

    async fn stat(&self, path: &str, _: OpStat) -> Result<RpStat> {
        console_log!("opfs_stat stat path = {path}!!!!!!!!!!!!!!!!!!!!!!!!");

        let rooted = build_rooted_abs_path(&self.root, path);
        let meta = opfs_stat(&rooted).await?;
        Ok(RpStat::new(meta))
    }

    async fn read(&self, path: &str, _args: OpRead) -> Result<(RpRead, Self::Reader)> {
        let rooted = build_rooted_abs_path(&self.root, path);
        let buf = opfs_read(&rooted).await?;
        Ok((RpRead::new(), OpfsReader::new(buf)))
    }

    async fn write(&self, path: &str, _op: OpWrite) -> Result<(RpWrite, Self::Writer)> {
        let rooted = build_rooted_abs_path(&self.root, path);
        Ok((RpWrite::default(), OpfsWriter::new(&rooted)))
    }

    async fn delete(&self) -> Result<(RpDelete, Self::Deleter)> {
        Ok((
            RpDelete::default(),
            oio::OneShotDeleter::new(OpfsDeleter {
                root: self.root.clone(),
            }),
        ))
    }

    async fn list(&self, path: &str, _: OpList) -> Result<(RpList, Self::Lister)> {
        let rooted = build_rooted_abs_path(&self.root, path);
        match opfs_list(&rooted).await {
            Ok(iterator) => {
                let lister = OpfsLister::new(path, iterator);
                Ok((RpList::default(), Some(lister)))
            }
            Err(e) if e.kind() == ErrorKind::NotFound => Ok((RpList::default(), None)),
            Err(e) => Err(e),
        }
    }
}
