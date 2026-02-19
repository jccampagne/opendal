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

pub fn console_debug(s: &impl std::fmt::Debug) {
    web_sys::console::log_1(&format!("value: {:?}", s).into())
    // web_sys::console::log_1(&format!("value: {:?}", res).into())
    // web_sys::console::log_1(.);
}

#[cfg(test)]
mod tests {
    use opendal::Operator;
    use opendal::services::OpfsConfig;
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;
    use web_sys::console::assert;

    use crate::console_debug;

    wasm_bindgen_test_configure!(run_in_browser);

    fn new_operator() -> Operator {
        Operator::from_config(OpfsConfig::default())
            .expect("failed to create opfs operator")
            .finish()
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    async fn test_write_read() {
        let op = new_operator();
        let filename = "test-write-read.txt";

        op.write(filename, "hello world").await.expect("write");
        op.write(filename, "hello world").await.expect("write");

        let data = op.read(filename).await.expect("read").to_bytes();
        assert_eq!(data.as_ref(), b"hello world");

        op.write(filename, "something else").await.expect("write");
        let data = op.read(filename).await.expect("read").to_bytes();
        assert_eq!(data.as_ref(), b"something else");

        op.write(filename, "").await.expect("write");
        let data = op.read(filename).await.expect("read").to_bytes();
        assert_eq!(data.as_ref(), b"");

        // delete is idempotent, Deleting a file that does not exist won’t return errors.
        // https://docs.rs/opendal/latest/opendal/struct.Operator.html#method.delete
        op.delete(filename).await.expect("delete");
        op.delete(filename).await.expect("delete");
        // console_debug(&r);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    async fn test_write_read_in_a_directory() {
        let op = new_operator();
        let filename = "/some/dir/test-write-read.txt";

        op.write(filename, "hello world").await.expect("write");
        op.write(filename, "hello world").await.expect("write");

        let data = op.read(filename).await.expect("read").to_bytes();
        assert_eq!(data.as_ref(), b"hello world");
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    async fn test_stat() {
        let op = new_operator();

        let filename = "test-stat.txt";
        op.write(filename, "some content").await.expect("write");

        let meta = op.stat(filename).await.unwrap();
        assert_eq!(meta.content_length(), 12);
        assert!(meta.last_modified().is_some());

        op.delete(filename).await.unwrap();
        let meta = op.stat(filename).await;

        assert!(meta.is_err());
        assert_eq!(meta.unwrap_err().kind(), opendal::ErrorKind::NotFound);
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    async fn test_create_dir_and_list() {
        let op = new_operator();

        op.create_dir("test-list/").await.unwrap();
        op.write("test-list/a.txt", "a").await.unwrap();
        op.write("test-list/b.txt", "b").await.unwrap();

        let entries = op.list("test-list/").await.unwrap();
        let mut names: Vec<String> = entries.iter().map(|e| e.path().to_string()).collect();
        names.sort();
        assert_eq!(names, vec!["test-list/a.txt", "test-list/b.txt"]);

        op.delete("test-list/a.txt").await.unwrap();
        op.delete("test-list/b.txt").await.unwrap();
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    async fn test_delete() {
        let op = new_operator();

        op.write("test-delete.txt", "bye").await.unwrap();
        op.stat("test-delete.txt").await.unwrap();

        op.delete("test-delete.txt").await.unwrap();

        let result = op.stat("test-delete.txt").await;
        assert!(result.is_err());
    }

    #[allow(dead_code)]
    #[wasm_bindgen_test]
    async fn test_stat_not_found() {
        let op = new_operator();

        let result = op.stat("nonexistent-file.txt").await;
        assert!(result.is_err());
    }
}
