// Copyright 2021 Datafuse Labs
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

//! Read-time checks of a materialized view against its source, read by table id.
//!
//! Source DDL writes the invalid reason of the MVs on a best-effort basis after it commits, so a
//! reason may be missing. Only final reasons are written: a SWAP of the source, and a change of a
//! column the MV reads. Readers and REFRESH check the source themselves. A dropped source (DROP
//! TABLE, or CREATE OR REPLACE TABLE, which drops the table it replaces) is rejected from its
//! `drop_on` and not recorded, so UNDROP TABLE makes the MV usable again. A changed column the MV
//! reads is recorded when the DDL left it out. A failure to bind the definition by the source
//! name is not recorded: it also follows a rename of the source or of its database, which leaves
//! the MV valid. A SWAP is not recorded either: it looks like a rename followed by a new table of
//! the name.

use std::collections::HashMap;

use databend_common_exception::ErrorCode;
use databend_common_exception::Result;
use databend_common_meta_api::TableApi;
use databend_common_meta_app::schema::MATERIALIZED_VIEW_SOURCE_DROPPED_REASON;
use databend_common_meta_app::schema::MATERIALIZED_VIEW_SOURCE_SCHEMA_CHANGED_REASON;
use databend_common_meta_app::schema::OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON;
use databend_common_meta_app::schema::TableMeta;
use databend_common_meta_app::schema::UpsertTableOptionReq;
use databend_common_users::UserApiProvider;
use databend_meta_client::types::MatchSeq;
use log::info;
use log::warn;

/// Best effort: set the invalid reason of MV `mv_id`. Only this option changes, on the latest
/// `TableMeta` of the MV, so a concurrent REFRESH commit is kept.
async fn record_invalid_reason(mv_id: u64, reason: &str) {
    info!(
        "record the invalid reason of materialized view {}: {:?}",
        mv_id, reason
    );
    if let Err(error) = UserApiProvider::instance()
        .get_meta_store_client()
        .upsert_table_option(UpsertTableOptionReq {
            table_id: mv_id,
            seq: MatchSeq::Any,
            options: HashMap::from([(
                OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON.to_string(),
                Some(reason.to_string()),
            )]),
        })
        .await
    {
        warn!(
            "failed to record the invalid reason of materialized view {}: {}",
            mv_id, error
        );
    }
}

/// The source of `source_meta`, read by id, unless it is missing or dropped.
fn active_source(source_meta: Option<&TableMeta>) -> Option<&TableMeta> {
    source_meta.filter(|meta| meta.drop_on.is_none())
}

/// Reject MV `mv_id`, named `mv_name` and with `mv_meta`, when it is invalid against its source,
/// `source_meta` as read by id: it has an invalid reason, the source is missing or dropped, or a
/// column the MV reads changed. Only the last is recorded, as the DDL may have left it out.
pub async fn check_materialized_view_source(
    mv_id: u64,
    mv_name: &str,
    mv_meta: &TableMeta,
    source_meta: Option<&TableMeta>,
) -> Result<()> {
    let invalid = |reason: &str| {
        // Only a dropped source can come back, by UNDROP TABLE.
        let hint = if reason == MATERIALIZED_VIEW_SOURCE_DROPPED_REASON {
            "undrop the source table or recreate the materialized view"
        } else {
            "recreate the materialized view"
        };
        ErrorCode::InvalidMaterializedView(format!(
            "materialized view {mv_name} is invalid: {reason}; {hint}"
        ))
    };
    if let Some(reason) = mv_meta.materialized_view_invalid_reason() {
        return Err(invalid(reason));
    }
    let Some(source_meta) = active_source(source_meta) else {
        return Err(invalid(MATERIALIZED_VIEW_SOURCE_DROPPED_REASON));
    };
    if mv_meta.materialized_view_source_schema_changed(&source_meta.schema)? {
        let reason = MATERIALIZED_VIEW_SOURCE_SCHEMA_CHANGED_REASON;
        record_invalid_reason(mv_id, reason).await;
        return Err(invalid(reason));
    }
    Ok(())
}
