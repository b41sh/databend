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

use std::collections::HashMap;

use common_arrow::parquet::metadata::ThriftFileMetaData;
use common_exception::ErrorCode;
use common_exception::Result;

use crate::storages::fuse::meta::ColumnId;
use crate::storages::fuse::meta::ColumnMeta;
use crate::storages::fuse::meta::ColumnSchema;

/**
#[derive(PartialEq)]
struct TreeNode {
    element: SchemaElement,
    name: String,
    column_id: Option<u32>,
    children: Option<Vec<TreeNode>>,
}

impl TreeNode {
    fn new(element: SchemaElement, name: String, column_id: Option<u32>) -> TreeNode {
        return TreeNode {
            element,
            column_id,
            children: None,
        };
    }

    fn add_child(&mut self, node: TreeNode) {
        if let Some(ref mut children) = self.children {
            children.push(node);
        } else {
            self.children = Some(vec![node]);
        }
    }
}
*/

pub fn column_metas(
    file_meta: &ThriftFileMetaData,
) -> Result<(HashMap<ColumnId, ColumnMeta>, Option<ColumnSchema>)> {
    println!("file_meta={:#?}", file_meta);

    // currently we use one group only
    let num_row_groups = file_meta.row_groups.len();
    if num_row_groups != 1 {
        return Err(ErrorCode::ParquetError(format!(
            "invalid parquet file, expects only one row group, but got {}",
            num_row_groups
        )));
    }

    /**
        let mut column_id = 0;
        let mut node_stack: std::vec::Vec<ColumnSchema> = Vec::new();
        let mut num_stack = Vec::new();
        // build tree struct from schema
        for element in file_meta.schema.iter() {
            let curr_column_id = match element.num_children {
                Some(_) => None,
                None => Some(column_id),
            };
            let node = ColumnSchema::new(element.name.clone(), curr_column_id);
            let len = node_stack.len();
            if len > 0 {
                let pnode = node_stack.get_mut(len - 1).unwrap();
                pnode.add_child(node.clone());
                let num = num_stack.pop().unwrap();
                if num > 0 {
                    num_stack.push(num-1);
                } else {
                    // keep root node in stack
                    if len > 1 {
                        node_stack.pop();
                    }
                }
            }
            if let Some(num_children) = element.num_children {
                node_stack.push(node);
                num_stack.push(num_children);
            } else {
                column_id += 1;
            }

            println!("node_stack={:?}", node_stack);
            println!("num_stack={:?}", num_stack);
        }
        let col_schema = node_stack.pop().unwrap();
    */
    let mut column_id = 0;
    let mut node_stack = Vec::new();
    let mut num_stack = Vec::new();
    // build tree struct from schema
    for element in file_meta.schema.iter() {
        let (curr_column_id, num_children) = match element.num_children {
            Some(num_children) => (None, num_children),
            None => (Some(column_id), 0),
        };
        num_stack.push(num_children);
        if element.num_children.is_none() {
            column_id += 1;
        }
        let node = ColumnSchema::new(element.name.clone(), curr_column_id);
        node_stack.push(node);
    }

    let mut node_stack2 = Vec::new();
    loop {
        if node_stack.is_empty() {
            break;
        }
        let num = num_stack.pop().unwrap();
        let mut node = node_stack.pop().unwrap();
        if num > 0 {
            for _ in 0..num {
                let child = node_stack2.pop().unwrap();
                node.add_child(child);
            }
        }
        node_stack2.push(node);

        println!("node_stack={:?}", node_stack);
        println!("num_stack={:?}", num_stack);

        println!("node_stack2={:?}", node_stack2);
        println!("-----------------------------------");
    }

    let col_schema = node_stack2.pop().unwrap();
    println!("col_schema={:?}", col_schema);

    let row_group = &file_meta.row_groups[0];
    let mut col_metas = HashMap::with_capacity(row_group.columns.len());
    for (idx, col_chunk) in row_group.columns.iter().enumerate() {
        match &col_chunk.meta_data {
            Some(chunk_meta) => {
                let col_start = if let Some(dict_page_offset) = chunk_meta.dictionary_page_offset {
                    dict_page_offset
                } else {
                    chunk_meta.data_page_offset
                };
                let col_len = chunk_meta.total_compressed_size;
                assert!(
                    col_start >= 0 && col_len >= 0,
                    "column start and length should not be negative"
                );
                let num_values = chunk_meta.num_values as u64;
                let res = ColumnMeta {
                    offset: col_start as u64,
                    len: col_len as u64,
                    num_values,
                };
                col_metas.insert(idx as u32, res);
            }
            None => {
                return Err(ErrorCode::ParquetError(format!(
                    "invalid parquet file, meta data of column idx {} is empty",
                    idx
                )))
            }
        }
    }
    Ok((col_metas, Some(col_schema)))
}
