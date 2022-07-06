// Copyright 2022 Datafuse Labs.
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

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use common_exception::ErrorCode;
use common_exception::Result;
use common_planners::PartInfo;
use common_planners::PartInfoPtr;

use crate::storages::fuse::meta::ColumnSchema;
use crate::storages::fuse::meta::Compression;

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
pub struct ColumnMeta {
    pub offset: u64,
    pub length: u64,
    pub num_values: u64,
}

impl ColumnMeta {
    pub fn create(offset: u64, length: u64, num_values: u64) -> ColumnMeta {
        ColumnMeta {
            offset,
            length,
            num_values,
        }
    }
}

/**
//1. 在 fields 里列的编号
//2. 在 parquet 里列的编号
//columns_info 的 key 用来表示在 fields 里的编号，value 里的 id 用来表示在 parquet 里的编号
//pushdowns 传下来的是 filed 的位置
#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq, Debug)]
pub struct ColumnInfo {
    /// The id
    pub field_id: u32,
    /// The column name
    pub name: String,
    ///
    pub path: Option<Vec<String>>,
    /// The optional child ids, used by Struct column and Variant column with sub column
    pub child_ids: Option<Vec<u32>>,
    /// The optional leaves id, to select column from parquet
    pub leaves_id: Option<u32>,
}

impl ColumnInfo {
    pub fn create(field_id: u32, name: String, path: Option<Vec<String>>, child_ids: Option<Vec<u32>>, leaves_id: Option<u32>) -> ColumnInfo {
        ColumnInfo {
            field_id,
            name,
            path,
            child_ids,
            leaves_id,
        }
    }
}
*/

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FusePartInfo {
    pub location: String,
    /// FusePartInfo itself is not versioned
    /// the `format_version` is the version of the block which the `location` points to
    pub format_version: u64,
    pub nums_rows: usize,
    pub columns_meta: HashMap<usize, ColumnMeta>,
    pub proj_map: HashMap<usize, Vec<usize>>,
    pub compression: Compression,
}

#[typetag::serde(name = "fuse")]
impl PartInfo for FusePartInfo {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn equals(&self, info: &Box<dyn PartInfo>) -> bool {
        match info.as_any().downcast_ref::<FusePartInfo>() {
            None => false,
            Some(other) => self == other,
        }
    }
}

impl FusePartInfo {
    pub fn create(
        location: String,
        format_version: u64,
        rows_count: u64,
        columns_meta: HashMap<usize, ColumnMeta>,
        proj_map: HashMap<u32, Vec<u32>>,
        compression: Compression,
    ) -> Arc<Box<dyn PartInfo>> {
        Arc::new(Box::new(FusePartInfo {
            location,
            format_version,
            columns_meta,
            proj_map,
            nums_rows: rows_count as usize,
            compression,
        }))
    }

    pub fn from_part(info: &PartInfoPtr) -> Result<&FusePartInfo> {
        match info.as_any().downcast_ref::<FusePartInfo>() {
            Some(part_ref) => Ok(part_ref),
            None => Err(ErrorCode::LogicalError(
                "Cannot downcast from PartInfo to FusePartInfo.",
            )),
        }
    }
}
