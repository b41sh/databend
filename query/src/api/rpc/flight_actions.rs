// Copyright 2021 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::convert::TryInto;

use common_arrow::arrow_format::flight::data::Action;
use common_exception::ErrorCode;
use common_exception::ToErrorCode;
use tonic::Status;

use crate::api::InitNodesChannelPacket;
use crate::api::QueryFragmentsPlanPacket;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CancelAction {
    pub query_id: String,
}

impl TryInto<CancelAction> for Vec<u8> {
    type Error = Status;

    fn try_into(self) -> Result<CancelAction, Self::Error> {
        match std::str::from_utf8(&self) {
            Err(cause) => Err(Status::invalid_argument(cause.to_string())),
            Ok(utf8_body) => match serde_json::from_str::<CancelAction>(utf8_body) {
                Err(cause) => Err(Status::invalid_argument(cause.to_string())),
                Ok(action) => Ok(action),
            },
        }
    }
}

impl TryInto<Vec<u8>> for CancelAction {
    type Error = ErrorCode;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        serde_json::to_vec(&self).map_err_to_code(ErrorCode::LogicalError, || {
            "Logical error: cannot serialize BroadcastAction."
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct InitQueryFragmentsPlan {
    pub executor_packet: QueryFragmentsPlanPacket,
}

impl TryInto<InitQueryFragmentsPlan> for Vec<u8> {
    type Error = Status;

    fn try_into(self) -> Result<InitQueryFragmentsPlan, Self::Error> {
        match std::str::from_utf8(&self) {
            Err(cause) => Err(Status::invalid_argument(cause.to_string())),
            Ok(utf8_body) => match serde_json::from_str::<InitQueryFragmentsPlan>(utf8_body) {
                Err(cause) => Err(Status::invalid_argument(cause.to_string())),
                Ok(action) => Ok(action),
            },
        }
    }
}

impl TryInto<Vec<u8>> for InitQueryFragmentsPlan {
    type Error = ErrorCode;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        serde_json::to_vec(&self).map_err_to_code(ErrorCode::LogicalError, || {
            "Logical error: cannot serialize PrepareExecutor."
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct InitNodesChannel {
    pub init_nodes_channel_packet: InitNodesChannelPacket,
}

impl TryInto<InitNodesChannel> for Vec<u8> {
    type Error = Status;

    fn try_into(self) -> Result<InitNodesChannel, Self::Error> {
        match std::str::from_utf8(&self) {
            Err(cause) => Err(Status::invalid_argument(cause.to_string())),
            Ok(utf8_body) => match serde_json::from_str::<InitNodesChannel>(utf8_body) {
                Err(cause) => Err(Status::invalid_argument(cause.to_string())),
                Ok(action) => Ok(action),
            },
        }
    }
}

impl TryInto<Vec<u8>> for InitNodesChannel {
    type Error = ErrorCode;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        serde_json::to_vec(&self).map_err_to_code(ErrorCode::LogicalError, || {
            "Logical error: cannot serialize PreparePublisher."
        })
    }
}

#[derive(Clone, Debug)]
pub enum FlightAction {
    CancelAction(CancelAction),
    InitQueryFragmentsPlan(InitQueryFragmentsPlan),
    InitNodesChannel(InitNodesChannel),
    ExecutePartialQuery(String),
}

impl TryInto<FlightAction> for Action {
    type Error = Status;

    fn try_into(self) -> Result<FlightAction, Self::Error> {
        match self.r#type.as_str() {
            "CancelAction" => Ok(FlightAction::CancelAction(self.body.try_into()?)),
            "InitQueryFragmentsPlan" => {
                Ok(FlightAction::InitQueryFragmentsPlan(self.body.try_into()?))
            }
            "InitNodesChannel" => Ok(FlightAction::InitNodesChannel(self.body.try_into()?)),
            "ExecutePartialQuery" => match String::from_utf8(self.body.to_owned()) {
                Ok(query_id) => Ok(FlightAction::ExecutePartialQuery(query_id)),
                Err(cause) => Err(Status::invalid_argument(cause.to_string())),
            },
            un_implemented => Err(Status::unimplemented(format!(
                "UnImplement action {}",
                un_implemented
            ))),
        }
    }
}

impl TryInto<Action> for FlightAction {
    type Error = ErrorCode;

    fn try_into(self) -> Result<Action, Self::Error> {
        match self {
            FlightAction::CancelAction(cancel_action) => Ok(Action {
                r#type: String::from("CancelAction"),
                body: cancel_action.try_into()?,
            }),
            FlightAction::InitQueryFragmentsPlan(init_query_fragments_plan) => Ok(Action {
                r#type: String::from("InitQueryFragmentsPlan"),
                body: init_query_fragments_plan.try_into()?,
            }),
            FlightAction::InitNodesChannel(init_nodes_channel) => Ok(Action {
                r#type: String::from("InitNodesChannel"),
                body: init_nodes_channel.try_into()?,
            }),
            FlightAction::ExecutePartialQuery(query_id) => Ok(Action {
                r#type: String::from("ExecutePartialQuery"),
                body: query_id.into_bytes(),
            }),
        }
    }
}
