//  Copyright 2022 Datafuse Labs.
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
//

use std::collections::HashMap;

use common_datablocks::DataBlock;
use serde::Deserialize;
use serde::Serialize;

use crate::storages::fuse::meta::common::ColumnId;
use crate::storages::fuse::meta::common::Compression;
use crate::storages::fuse::meta::common::FormatVersion;
use crate::storages::fuse::meta::common::Location;
use crate::storages::fuse::meta::common::Statistics;
use crate::storages::fuse::meta::common::Versioned;
use crate::storages::fuse::meta::v0::ColumnMeta;
use crate::storages::index::ClusterStatistics;
use crate::storages::index::ColumnStatistics;

/// A segment comprises one or more blocks
#[derive(Serialize, Deserialize, Debug)]
pub struct SegmentInfo {
    /// format version
    format_version: FormatVersion,
    /// blocks belong to this segment
    pub blocks: Vec<BlockMeta>,
    /// summary statistics
    pub summary: Statistics,
}

/**
#[derive(Serialize, Deserialize, Debug)]
pub struct ColumnInfo {
    /// The field id
    pub field_id: u32,
    /// The column name
    pub name: String,
    ///
    pub path: Option<Vec<String>>,
    /// The optional child ids, used by Struct column and Variant column with sub column
    pub child_ids: Option<Vec<ColumnInfo>>,
    /// The optional leaves id, to select column from parquet
    pub leaves_id: Option<u32>,
}
*/

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct ColumnSchema {
    pub name: String,
    pub column_id: Option<u32>,
    pub children: Option<Vec<ColumnSchema>>,
}

impl ColumnSchema {
    pub fn new(name: String, column_id: Option<u32>) -> ColumnSchema {
        return ColumnSchema {
            name,
            column_id,
            children: None,
        };
    }

    pub fn add_child(&mut self, child: ColumnSchema) {
        if let Some(ref mut children) = self.children {
            children.push(child);
        } else {
            self.children = Some(vec![child]);
        }
    }
}



/// Meta information of a block
/// Part of and kept inside the [SegmentInfo]
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockMeta {
    pub row_count: u64,
    pub block_size: u64,
    pub file_size: u64,
    pub col_stats: HashMap<ColumnId, ColumnStatistics>,
    pub col_metas: HashMap<ColumnId, ColumnMeta>,
    pub col_schema: Option<ColumnSchema>,
    pub cluster_stats: Option<ClusterStatistics>,
    pub location: Location,

    /// Compression algo used to compress the columns of blocks
    ///
    /// If not specified, the legacy algo `Lz4` will be used.
    /// `Lz4` is merely for backward compatibility, it will NO longer be
    /// used in the write path.
    #[serde(default = "Compression::legacy")]
    compression: Compression,
}

impl BlockMeta {
    pub fn new(
        row_count: u64,
        block_size: u64,
        file_size: u64,
        col_stats: HashMap<ColumnId, ColumnStatistics>,
        col_metas: HashMap<ColumnId, ColumnMeta>,
        col_schema: Option<ColumnSchema>,
        cluster_stats: Option<ClusterStatistics>,
        location: Location,
    ) -> Self {
        Self {
            row_count,
            block_size,
            file_size,
            col_stats,
            col_metas,
            col_schema,
            cluster_stats,
            location,
            compression: Compression::Lz4Raw,
        }
    }

    pub fn compression(&self) -> Compression {
        self.compression
    }
}

impl SegmentInfo {
    pub fn new(blocks: Vec<BlockMeta>, summary: Statistics) -> Self {
        Self {
            format_version: SegmentInfo::VERSION,
            blocks,
            summary,
        }
    }

    pub fn format_version(&self) -> u64 {
        self.format_version
    }
}

use super::super::v0;

impl From<v0::SegmentInfo> for SegmentInfo {
    fn from(s: v0::SegmentInfo) -> Self {
        Self {
            format_version: SegmentInfo::VERSION,
            blocks: s.blocks.into_iter().map(|b| b.into()).collect::<_>(),
            summary: s.summary,
        }
    }
}

impl From<v0::BlockMeta> for BlockMeta {
    fn from(s: v0::BlockMeta) -> Self {
        Self {
            row_count: s.row_count,
            block_size: s.block_size,
            file_size: s.file_size,
            col_stats: s.col_stats,
            col_metas: s.col_metas,
            col_schema: None,
            cluster_stats: None,
            location: (s.location.path, DataBlock::VERSION),
            compression: Compression::Lz4,
        }
    }
}
