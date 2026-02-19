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

use opendal::Builder;
use opendal::Configurator;
use opendal::Operator;
use opendal::services::OpfsConfig;
use wasm_bindgen::prelude::*;

pub fn console_debug(s: &impl Debug) {
    web_sys::console::log_1(&format!("value: {:?}", s).into())
    // web_sys::console::log_1(&format!("value: {:?}", res).into())
    // web_sys::console::log_1(.);
}
#[wasm_bindgen]
pub async fn hello_world() -> String {
    let opfs_config = OpfsConfig::default();

    // let op: Operator = Operator::from_config(OpfsConfig::default())?.finish();
    let op = Operator::from_config(OpfsConfig::default())
        .expect("could not create opfs operator")
        .finish();
    let info = op.info();

    let mut s = format!("info = {:?}\n", info);
    let res = op.create_dir("/test/").await;
    s += &format!("res = {:?}'n", res);
    console_debug(&"create_dir");
    console_debug(&res);

    let res = op.write("my-file.txt", "hello world").await;
    s += &format!("res = {:?}'n", res);
    console_debug(&"write");
    console_debug(&res);

    // op.write("my-file.txt", "hello world").await.unwrap();
    let data = op.read("my-file.txt").await.unwrap().to_bytes();
    let text = String::from_utf8_lossy(&data);
    console_debug(&text);

    let meta = op.stat("my-file.txt").await.unwrap();
    console_debug(&format!(
        "stat: size={}, last_modified={:?}",
        meta.content_length(),
        meta.last_modified()
    ));

    let mut entries = op.list("/").await.unwrap();
    let names: Vec<String> = entries.iter().map(|e| e.path().to_string()).collect();
    console_debug(&format!("entries: {:?}", names));

    {
        let filename_for_delete = "to-delete.txt";
        let meta = op.stat(filename_for_delete).await;
        console_debug(&format!("meta: {:?}", meta));

        op.write(filename_for_delete, "bye").await;

        let meta = op.stat(filename_for_delete).await;
        console_debug(&format!("meta: {:?}", meta));

        op.delete(filename_for_delete).await;
        let meta = op.stat(filename_for_delete).await;
        console_debug(&format!("meta: {:?}", meta));
    }
    s

    // let opfs_builder = opfs_config.into_builder();
    // let maybe_opfs = opfs_builder.build();
    // match maybe_opfs {
    //     Ok(opfs) => opfs.build().await,
    //     Err(_) => todo!(),
    // }
    // let op = Operator::new(cfg).unwrap().finish();
    // op.write(
    //     "test",
    //     "Hello, WASM! We are from OpenDAL at rust side!"
    //         .as_bytes()
    //         .to_vec(),
    // )
    // .await
    // .unwrap();
    // let bs = op.read("test").await.unwrap().to_bytes();
    // String::from_utf8_lossy(&bs).to_string()
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;

    use super::*;

    wasm_bindgen_test_configure!(run_in_browser);

    /// This test is executed in the wasm target but rustc reports `dead_code` in wrong.
    #[allow(dead_code)]
    #[wasm_bindgen_test]
    async fn test_hello_world() {
        let s = hello_world().await;
        assert_eq!(s, "Hello, WASM! We are from OpenDAL at rust side!")
    }
}
