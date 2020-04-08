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

// Insert std prelude in the top for the sgx feature
#[cfg(feature = "mesalock_sgx")]
use std::prelude::v1::*;

use kms_proto::proto::*;
use mesatee_core::config::{OutboundDesc, TargetDesc};
use mesatee_core::rpc::channel::SgxTrustedChannel;
use mesatee_core::Result;

pub struct KMSClient {
    user_id: String,
    user_token: String,
    channel: SgxTrustedChannel<KMSRequest, KMSResponse>,
}

impl KMSClient {
    pub fn new(target: &TargetDesc, user_id: &str, user_token: &str) -> Result<Self> {
        let addr = target.addr;
        let channel = match &target.desc {
            OutboundDesc::Sgx(enclave_attr) => SgxTrustedChannel::<
                KMSRequest,
                KMSResponse,
            >::new(addr, enclave_attr.clone())?,
        };
        Ok(KMSClient { 
	    user_id: user_id.to_string(),
	    user_token: user_token.to_string(), 
	    channel: channel,
         })
    }

    pub fn create_key(
        &mut self,
    ) -> Result<CreateKeyResponse> {
        //TODO handle authentication
        let req = CreateKeyRequest::new(kms_proto::EncType::ProtectedFs);
        let resp = self.channel.invoke(KMSRequest::CreateKey(req))?;
        match resp {
	    KMSResponse::CreateKey(resp) => Ok(resp),
	    _ => Err(mesatee_core::Error::from(mesatee_core::ErrorKind::RPCResponseError)),
	}
    }
}
