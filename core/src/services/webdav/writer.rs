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

use async_trait::async_trait;
use http::StatusCode;

use super::backend::WebdavBackend;
use super::error::parse_error;
use crate::raw::oio::WriteBuf;
use crate::raw::*;
use crate::*;

pub struct WebdavWriter {
    backend: WebdavBackend,

    op: OpWrite,
    path: String,
}

impl WebdavWriter {
    pub fn new(backend: WebdavBackend, op: OpWrite, path: String) -> Self {
        WebdavWriter { backend, op, path }
    }
}

#[async_trait]
impl oio::OneShotWrite for WebdavWriter {
    async fn write_once(&self, bs: &dyn WriteBuf) -> Result<()> {
        let bs = oio::ChunkedBytes::from_vec(bs.vectored_bytes(bs.remaining()));

        let resp = self
            .backend
            .webdav_put(
                &self.path,
                Some(bs.len() as u64),
                &self.op,
                AsyncBody::ChunkedBytes(bs),
            )
            .await?;

        let status = resp.status();

        match status {
            StatusCode::CREATED | StatusCode::OK | StatusCode::NO_CONTENT => {
                resp.into_body().consume().await?;
                Ok(())
            }
            _ => Err(parse_error(resp).await?),
        }
    }
}
