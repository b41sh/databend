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

use std::collections::BTreeSet;
use std::collections::HashMap;
use std::sync::Arc;

use databend_common_catalog::table::Table;
use databend_common_catalog::table::TableExt;
use databend_common_exception::ErrorCode;
use databend_common_exception::Result;
use databend_common_expression::TableSchema;
use databend_common_license::license::Feature;
use databend_common_license::license_manager::LicenseManagerSwitch;
use databend_common_meta_api::MaterializedViewApi;
use databend_common_meta_api::TableApi;
use databend_common_meta_app::schema::GetTableReq;
use databend_common_meta_app::schema::ListedMVSource;
use databend_common_meta_app::schema::MVInvalidReason;
use databend_common_meta_app::schema::MVSourceColumn;
use databend_common_meta_app::schema::MaterializedViewListFilter;
use databend_common_meta_app::schema::OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON;
use databend_common_meta_app::schema::OPT_KEY_MATERIALIZED_VIEW_SOURCE_COLUMNS;
use databend_common_meta_app::schema::OPT_KEY_MATERIALIZED_VIEW_SOURCE_DATABASE_NAME;
use databend_common_meta_app::schema::OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_NAME;
use databend_common_meta_app::schema::TableMeta;
use databend_common_meta_app::schema::UpsertTableOptionReq;
use databend_common_meta_app::schema::is_materialized_view_engine;
use databend_common_meta_app::tenant::Tenant;
use databend_common_meta_store::MetaStore;
use databend_common_sql::plans::MaintenanceTarget;
use databend_common_users::UserApiProvider;
use databend_meta_client::types::MatchSeq;
use log::info;
use log::warn;

use crate::meta_service_error;
use crate::sessions::QueryContext;
use crate::sessions::TableContextLicense;
use crate::sessions::TableContextTableAccess;

pub fn check_materialized_view_license(ctx: &QueryContext, engine: &str) -> Result<()> {
    if is_materialized_view_engine(engine) {
        LicenseManagerSwitch::instance()
            .check_enterprise_enabled(ctx.get_license_key(), Feature::MaterializedView)?;
    }
    Ok(())
}

/// Reject attaching a row access or masking policy to a materialized-view source.
///
/// MV storage is computed without reader policies, so a source with any dependent MV, including
/// an invalid one, cannot get a policy. Drop the MVs first.
pub async fn check_policy_target_without_materialized_view(
    ctx: &QueryContext,
    table: &dyn Table,
) -> Result<()> {
    if !UserApiProvider::instance()
        .get_meta_store_client()
        .list_mvs_by_source_table_id(&ctx.get_tenant(), table.get_id())
        .await
        .map_err(meta_service_error)?
        .is_empty()
    {
        return Err(ErrorCode::AlterTableError(format!(
            "Cannot attach a row access or masking policy to table '{}' because one or more materialized views depend on it; drop the materialized views first",
            table.name()
        )));
    }
    Ok(())
}

/// The state of a source table that decides the invalid reason of its materialized views.
enum SourceState {
    /// The table is dropped, replaced, or its metadata is gone.
    Dropped,
    /// The database holding the table is dropped.
    DatabaseDropped,
    Active {
        database_name: String,
        table_name: String,
        schema: Arc<TableSchema>,
    },
}

async fn source_state(
    meta: &MetaStore,
    tenant: &Tenant,
    source: &ListedMVSource,
) -> Result<SourceState> {
    if !source.is_active {
        return Ok(if source.database_dropped {
            SourceState::DatabaseDropped
        } else {
            SourceState::Dropped
        });
    }
    let (Some(database_name), Some(table_name)) = (&source.database_name, &source.table_name)
    else {
        return Ok(SourceState::Dropped);
    };
    // CREATE OR REPLACE TABLE ... AS SELECT points the name at the new table without marking the
    // replaced one dropped: a table its name no longer leads to is gone for its MVs.
    let table = match meta
        .get_table(GetTableReq::new(tenant, database_name, table_name))
        .await
    {
        Ok(table) => table,
        Err(error) => {
            let error = ErrorCode::from(error);
            if error.code() == ErrorCode::UNKNOWN_TABLE {
                return Ok(SourceState::Dropped);
            }
            return Err(error);
        }
    };
    if table.ident.table_id != source.table_id {
        return Ok(SourceState::Dropped);
    }
    Ok(SourceState::Active {
        database_name: database_name.clone(),
        table_name: table_name.clone(),
        schema: table.meta.schema.clone(),
    })
}

/// The invalid reason `state` calls for an MV with `table_meta`. A schema change has priority
/// over a database rename, which has priority over a table rename.
fn expected_reason(
    mv_id: u64,
    table_meta: &TableMeta,
    state: &SourceState,
) -> Option<MVInvalidReason> {
    match state {
        SourceState::Dropped => Some(MVInvalidReason::SourceDropped),
        SourceState::DatabaseDropped => Some(MVInvalidReason::SourceDatabaseDropped),
        SourceState::Active {
            database_name,
            table_name,
            schema,
        } => {
            let schema_changed = table_meta
                .materialized_view_source_schema_changed(schema)
                .unwrap_or_else(|error| {
                    warn!(
                        "materialized view {} has unreadable source columns, schema not checked: {}",
                        mv_id, error
                    );
                    false
                });
            let bound = |key: &str| table_meta.options.get(key).map(String::as_str);
            if schema_changed {
                Some(MVInvalidReason::SourceSchemaChanged)
            } else if bound(OPT_KEY_MATERIALIZED_VIEW_SOURCE_DATABASE_NAME)
                != Some(database_name.as_str())
            {
                Some(MVInvalidReason::SourceDatabaseRenamed)
            } else if bound(OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_NAME)
                != Some(table_name.as_str())
            {
                Some(MVInvalidReason::SourceRenamed)
            } else {
                None
            }
        }
    }
}

/// The change of the invalid reason option that brings `table_meta` to `expected`, or `None`
/// when there is nothing to change. A schema change is final; lifecycle reasons are reversible:
/// UNDROP and a RENAME back to the name the MV was created against clear them.
fn reason_change(
    table_meta: &TableMeta,
    expected: Option<MVInvalidReason>,
) -> Option<Option<String>> {
    let current = table_meta.materialized_view_invalid_reason();
    if current == Some(MVInvalidReason::SourceSchemaChanged.as_str()) {
        return None;
    }
    match expected {
        Some(reason) if current != Some(reason.as_str()) => Some(Some(reason.as_str().to_string())),
        None if table_meta.materialized_view_invalid_reason_restorable() => Some(None),
        _ => None,
    }
}

/// List the materialized views of `source_table_ids`, or of the tenant, and set or clear the
/// invalid reason of each to match the current state of its source. Every update only touches
/// the option on the latest `TableMeta` of the MV, so it does not lose a concurrent REFRESH
/// commit.
async fn reconcile(ctx: &QueryContext, source_table_ids: Option<BTreeSet<u64>>) -> Result<()> {
    let meta = UserApiProvider::instance().get_meta_store_client();
    let tenant = ctx.get_tenant();
    let mvs = meta
        .list_materialized_views(&tenant, &MaterializedViewListFilter {
            source_table_ids,
            ..Default::default()
        })
        .await
        .map_err(meta_service_error)?;
    let mut states = HashMap::new();
    for mv in mvs {
        let source_table_id = mv.source.table_id;
        if !states.contains_key(&source_table_id) {
            let state = source_state(&meta, &tenant, &mv.source)
                .await
                .inspect_err(|error| {
                    warn!(
                        "failed to read source table {} of materialized views, their invalid reason may be stale: {}",
                        source_table_id, error
                    );
                })
                .ok();
            states.insert(source_table_id, state);
        }
        let Some(state) = &states[&source_table_id] else {
            continue;
        };
        let mv_id = mv.mv.mv_id;
        let table_meta = &mv.mv.table_meta.data;
        let expected = expected_reason(mv_id, table_meta, state);
        let Some(reason) = reason_change(table_meta, expected) else {
            continue;
        };
        info!(
            "set the invalid reason of materialized view {} with source table {} to {:?}",
            mv_id, source_table_id, reason
        );
        if let Err(error) = meta
            .upsert_table_option(UpsertTableOptionReq {
                table_id: mv_id,
                seq: MatchSeq::Any,
                options: HashMap::from([(
                    OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON.to_string(),
                    reason,
                )]),
            })
            .await
        {
            warn!(
                "failed to set the invalid reason of materialized view {}: {}",
                mv_id, error
            );
        }
    }
    Ok(())
}

/// Best effort, after a DDL that may change the identity or the schema of `source_table_ids`:
/// bring the invalid reason of their materialized views in line with the sources.
///
/// The reason is not written in the DDL's transaction, so that the DDL does not grow with the
/// MVs of its tables. A failure is logged and does not fail the DDL: MV readers and REFRESH check
/// the source identity and schema themselves, and the next DDL of the source repairs the reason.
pub async fn reconcile_materialized_views(ctx: &QueryContext, source_table_ids: &[u64]) {
    if source_table_ids.is_empty() {
        return;
    }
    let ids = source_table_ids.iter().copied().collect::<BTreeSet<_>>();
    if let Err(error) = reconcile(ctx, Some(ids)).await {
        warn!(
            "failed to reconcile the materialized views of source tables {:?}, their invalid reason may be stale: {}",
            source_table_ids, error
        );
    }
}

/// [`reconcile_materialized_views`] for every materialized view of the tenant, after a database
/// DDL: the MVs are few, and the tables of the database need not be listed for it.
pub async fn reconcile_all_materialized_views(ctx: &QueryContext) {
    if let Err(error) = reconcile(ctx, None).await {
        warn!(
            "failed to reconcile the materialized views, their invalid reason may be stale: {}",
            error
        );
    }
}

/// Best effort, before a schema change of `table`: record its current columns in its
/// materialized views created by an older version, which did not record them. Without them the
/// views cannot tell whether a later schema change reached a column they read. A failure is
/// logged and does not fail the DDL.
pub async fn backfill_materialized_view_source_columns(ctx: &QueryContext, table: &dyn Table) {
    if table.is_temp() {
        return;
    }
    let table_id = table.get_id();
    let meta = UserApiProvider::instance().get_meta_store_client();
    let mvs = match meta
        .list_materialized_views(&ctx.get_tenant(), &MaterializedViewListFilter {
            source_table_ids: Some(BTreeSet::from([table_id])),
            ..Default::default()
        })
        .await
    {
        Ok(mvs) => mvs,
        Err(error) => {
            warn!(
                "failed to list the materialized views of source table {}, a schema change may not invalidate them: {}",
                table_id, error
            );
            return;
        }
    };
    let mut source_columns = None;
    for mv in mvs {
        if mv
            .mv
            .table_meta
            .data
            .options
            .contains_key(OPT_KEY_MATERIALIZED_VIEW_SOURCE_COLUMNS)
        {
            continue;
        }
        let source_columns = source_columns.get_or_insert_with(|| {
            MVSourceColumn::encode(&MVSourceColumn::from_schema(
                &table.get_table_info().meta.schema,
            ))
        });
        let mv_id = mv.mv.mv_id;
        if let Err(error) = meta
            .upsert_table_option(UpsertTableOptionReq {
                table_id: mv_id,
                seq: MatchSeq::Any,
                options: HashMap::from([(
                    OPT_KEY_MATERIALIZED_VIEW_SOURCE_COLUMNS.to_string(),
                    Some(source_columns.clone()),
                )]),
            })
            .await
        {
            warn!(
                "failed to record the source columns of materialized view {} from source table {}, a schema change may not invalidate it: {}",
                mv_id, table_id, error
            );
        }
    }
}

pub fn check_maintenance_target(table: &dyn Table, target: &MaintenanceTarget) -> Result<()> {
    match target {
        MaintenanceTarget::Table => table.check_mutable(),
        MaintenanceTarget::MaterializedView { table_id } => {
            if table.get_id() != *table_id || !is_materialized_view_engine(table.engine()) {
                return Err(ErrorCode::InvalidOperation(format!(
                    "Materialized view maintenance target changed: expected table id {}, found '{}'(id {}, engine {})",
                    table_id,
                    table.name(),
                    table.get_id(),
                    table.engine()
                )));
            }
            Ok(())
        }
    }
}
