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
//! reason may be missing or stale. Readers and REFRESH check the source themselves, and record
//! the reason when its cause is certain: the source is dropped (DROP TABLE, or CREATE OR REPLACE
//! TABLE, which drops the table it replaces), it was undropped, or a column the MV reads changed.
//! A failure to bind the definition by the source name is not recorded: it also follows a rename
//! of the source or of its database, which leaves the MV valid. A SWAP is not recorded either: it
//! looks like a rename followed by a new table of the name.

use std::collections::HashMap;

use databend_common_exception::ErrorCode;
use databend_common_exception::Result;
use databend_common_meta_api::TableApi;
use databend_common_meta_app::schema::MVInvalidReason;
use databend_common_meta_app::schema::OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON;
use databend_common_meta_app::schema::TableMeta;
use databend_common_meta_app::schema::UpsertTableOptionReq;
use databend_common_users::UserApiProvider;
use databend_meta_client::types::MatchSeq;
use log::info;
use log::warn;

/// Best effort: set (`Some`) or clear (`None`) the invalid reason of MV `mv_id`. Only this option
/// changes, on the latest `TableMeta` of the MV, so a concurrent REFRESH commit is kept.
async fn record_invalid_reason(mv_id: u64, reason: Option<MVInvalidReason>) {
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
                reason.map(|reason| reason.as_str().to_string()),
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

/// Clear a `source table was dropped` reason of MV `mv_id`, with `mv_meta`, whose source,
/// `source_meta`, is no longer dropped: UNDROP TABLE restored it but did not clear the reason.
/// Returns `mv_meta` without the reason when it does; the MV is valid again even if recording
/// the change fails.
pub async fn restore_undropped_materialized_view(
    mv_id: u64,
    mv_meta: &TableMeta,
    source_meta: Option<&TableMeta>,
) -> Option<TableMeta> {
    if active_source(source_meta).is_none()
        || !mv_meta.materialized_view_invalid_reason_restorable()
    {
        return None;
    }
    record_invalid_reason(mv_id, None).await;
    let mut restored = mv_meta.clone();
    restored
        .options
        .remove(OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON);
    Some(restored)
}

/// Reject MV `mv_id`, named `mv_name` and with `mv_meta`, when it is invalid against its source,
/// `source_meta` as read by id. A missing reason whose cause is certain, a dropped source or a
/// change of a column the MV reads, is recorded. A dropped reason that UNDROP left behind is
/// not cleared here, see [`restore_undropped_materialized_view`].
pub async fn check_materialized_view_source(
    mv_id: u64,
    mv_name: &str,
    mv_meta: &TableMeta,
    source_meta: Option<&TableMeta>,
) -> Result<()> {
    let invalid = |reason: &str| {
        ErrorCode::InvalidMaterializedView(format!(
            "materialized view {mv_name} is invalid: {reason}; recreate the materialized view"
        ))
    };
    if let Some(reason) = mv_meta.materialized_view_invalid_reason() {
        return Err(invalid(reason));
    }
    let Some(source_meta) = active_source(source_meta) else {
        let reason = MVInvalidReason::SourceDropped;
        record_invalid_reason(mv_id, Some(reason)).await;
        return Err(invalid(reason.as_str()));
    };
    if mv_meta.materialized_view_source_schema_changed(&source_meta.schema)? {
        let reason = MVInvalidReason::SourceSchemaChanged;
        record_invalid_reason(mv_id, Some(reason)).await;
        return Err(invalid(reason.as_str()));
    }
    Ok(())
}
