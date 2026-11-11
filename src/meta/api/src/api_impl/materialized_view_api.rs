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

//! An MV reuses ordinary `TableMeta`; its table ID is also its MV ID.
//!
//! ```text
//! __fd_table_by_id/<mv_id>
//!     -> TableMeta
//!        options["materialized_view_source_table_id"] = <source_id>
//!        options["materialized_view_source_database_name"] = <bound db name>
//!        options["materialized_view_source_table_name"] = <bound table name>
//!        options["materialized_view_invalid_reason"] = <reason>, while invalid
//! __fd_materialized_view_definition/<tenant>/<mv_id>
//!     -> MVDefinition { query, logical_schema }
//! __fd_materialized_view_by_source/<tenant>/<source_id>/<mv_id>
//!     -> MVSourceBinding (no state)
//! ```
//!
//! Whether an MV is valid is stored explicitly in its `TableMeta` options.
//! Source DDL that breaks the binding finds the dependent MVs through the
//! `SourceTableMVIdent` edges and writes the reason into each MV `TableMeta` in
//! the DDL's own transaction, with an exact-sequence condition on each. A
//! concurrent REFRESH commit of the same MV fails that condition; the DDL
//! retries. Readers decide validity from the `TableMeta` they read anyway; no
//! other record is compared. The edge is kept after invalidation so
//! management, restore, and GC can still find the MV.
//!
//! Lifecycle reasons are reversible: UNDROP and a RENAME back to the database
//! and table name the MV was bound to clear them (`restore_mvs_of_source`). A
//! schema change is final and overrides a lifecycle reason; a lifecycle reason
//! never overrides a schema change.
//!
//! The metadata operations are:
//!
//! ```text
//! CREATE MV txn:
//!     assert the source TableMeta exists and is not dropped
//!     put TableId(new_mv_id) = new_mv_table_meta
//!     put MVDefinitionIdent(tenant, new_mv_id) = definition
//!     put SourceTableMVIdent(tenant, source_id, new_mv_id)
//!
//! REPLACE MV txn:
//!     mark TableId(old_mv_id) dropped
//!     delete MVDefinitionIdent(tenant, old_mv_id)
//!     delete SourceTableMVIdent(tenant, old_source_id, old_mv_id)
//!     append the CREATE MV operations for new_mv_id
//!
//! DROP MV txn:
//!     mark TableId(mv_id) dropped
//!     delete MVDefinitionIdent(tenant, mv_id)
//!     delete SourceTableMVIdent(tenant, source_id, mv_id)
//!
//! RENAME/DROP/MODIFY COLUMN ON SOURCE txn:
//!     update the source TableMeta using its exact seq as CAS
//!     mark_mvs_invalid(source_id, SourceSchemaChanged)
//!
//! ADD COLUMN ON SOURCE txn:
//!     update the source TableMeta
//!     no MV metadata changes
//!
//! RENAME SOURCE TABLE txn:
//!     update the source table-name mappings
//!     new name == bound name: restore_mvs_of_source(source_id)
//!     otherwise:              mark_mvs_invalid(source_id, SourceRenamed)
//!
//! RENAME DATABASE txn:
//!     update the database-name mappings
//!     for each table of the database, as RENAME SOURCE TABLE with SourceDatabaseRenamed
//!
//! DROP SOURCE TABLE txn:
//!     update the ordinary source-table lifecycle metadata
//!     mark_mvs_invalid(source_id, SourceDropped)
//!
//! UNDROP SOURCE TABLE txn:
//!     update the ordinary source-table lifecycle metadata
//!     restore_mvs_of_source(source_id)
//!
//! DROP DATABASE txn:
//!     mark the DatabaseMeta dropped
//!     for each table of the database: mark_mvs_invalid(table_id, SourceDatabaseDropped)
//!
//! UNDROP DATABASE txn:
//!     restore the DatabaseMeta
//!     for each active table of the database: restore_mvs_of_source(table_id)
//!
//! GC MV txn:
//!     delete MVDefinitionIdent(tenant, mv_id)
//!     delete SourceTableMVIdent(tenant, source_id, mv_id)
//!     delete the ordinary table metadata
//!
//! GC source txn:
//!     delete SourceTableMVIdent(tenant, source_id, *)
//! ```

use std::collections::BTreeSet;
use std::collections::HashMap;

use databend_common_meta_app::schema::DBIdTableName;
use databend_common_meta_app::schema::DatabaseId;
use databend_common_meta_app::schema::DatabaseIdToName;
use databend_common_meta_app::schema::ListedMVSource;
use databend_common_meta_app::schema::ListedMaterializedView;
use databend_common_meta_app::schema::MVDefinition;
use databend_common_meta_app::schema::MVDefinitionIdent;
use databend_common_meta_app::schema::MVInfo;
use databend_common_meta_app::schema::MVInvalidReason;
use databend_common_meta_app::schema::MVSourceBindingSnapshot;
use databend_common_meta_app::schema::MaterializedViewListFilter;
use databend_common_meta_app::schema::OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON;
use databend_common_meta_app::schema::OPT_KEY_MATERIALIZED_VIEW_SOURCE_DATABASE_NAME;
use databend_common_meta_app::schema::OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_NAME;
use databend_common_meta_app::schema::SourceTableMV;
use databend_common_meta_app::schema::SourceTableMVIdent;
use databend_common_meta_app::schema::TableId;
use databend_common_meta_app::schema::TableIdToName;
use databend_common_meta_app::schema::TableMeta;
use databend_common_meta_app::schema::database_name_ident::DatabaseNameIdent;
use databend_common_meta_app::schema::is_materialized_view_engine;
use databend_common_meta_app::tenant::Tenant;
use databend_meta_client::kvapi;
use databend_meta_client::kvapi::DirName;
use databend_meta_client::kvapi::KvApiExt;
use databend_meta_client::kvapi::ListOptions;
use databend_meta_client::kvapi::StructKey;
use databend_meta_client::types::MetaError;
use databend_meta_client::types::SeqV;
use databend_meta_client::types::TxnGetResponse;
use databend_meta_client::types::TxnRequest;
use databend_meta_client::types::protobuf as pb;
use log::info;
use log::warn;

use crate::DEFAULT_MGET_SIZE;
use crate::deserialize_struct_get_response;
use crate::error_util::invalid_reply;
use crate::kv_pb_api::KVPbApi;
use crate::txn_cond_eq_seq;
use crate::txn_put_pb;

/// APIs for metadata that belongs exclusively to materialized views.
#[async_trait::async_trait]
pub trait MaterializedViewApi
where
    Self: Send + Sync,
    Self: kvapi::KVApi<Error = MetaError>,
{
    #[logcall::logcall]
    #[fastrace::trace]
    async fn get_mv_definition(
        &self,
        tenant: &Tenant,
        mv_table_id: u64,
    ) -> Result<Option<SeqV<MVDefinition>>, MetaError> {
        let ident = MVDefinitionIdent::new(tenant, mv_table_id);
        self.get_pb(&ident).await
    }

    /// List the valid MVs of a source table, for query rewrite.
    #[logcall::logcall]
    #[fastrace::trace]
    async fn get_mv_source_binding_snapshot(
        &self,
        tenant: &Tenant,
        source_table_id: u64,
    ) -> Result<MVSourceBindingSnapshot, MetaError> {
        let mut materialized_views =
            list_mvs_by_source_table_id_impl(self, tenant, source_table_id).await?;
        materialized_views.retain(|mv| mv.table_meta.data.is_valid_materialized_view());
        Ok(MVSourceBindingSnapshot { materialized_views })
    }

    /// List every MV that depends on a source table, including invalid MVs.
    ///
    /// This unfiltered view is intended for management, SHOW, and GC.
    #[logcall::logcall]
    #[fastrace::trace]
    async fn list_mvs_by_source_table_id(
        &self,
        tenant: &Tenant,
        source_table_id: u64,
    ) -> Result<Vec<MVInfo>, MetaError> {
        list_mvs_by_source_table_id_impl(self, tenant, source_table_id).await
    }

    /// List materialized views for management and system-table consumers.
    #[logcall::logcall]
    #[fastrace::trace]
    async fn list_materialized_views(
        &self,
        tenant: &Tenant,
        filter: &MaterializedViewListFilter,
    ) -> Result<Vec<ListedMaterializedView>, MetaError> {
        list_materialized_views_impl(self, tenant, filter).await
    }
}

/// Add to `txn` the operations that mark every MV of `source_table_id` invalid with `reason`.
///
/// Each MV `TableMeta` is updated under its exact sequence, so a concurrent change to the same
/// MV (a REFRESH commit, another DDL) fails the caller's transaction, which retries. An MV
/// already holding `SourceSchemaChanged` keeps it: that reason is final. An MV already holding
/// `reason` is left alone.
pub async fn mark_mvs_invalid<KV>(
    kv_api: &KV,
    txn: &mut TxnRequest,
    tenant: &Tenant,
    source_table_id: u64,
    reason: MVInvalidReason,
) -> Result<(), MetaError>
where
    KV: kvapi::KVApi<Error = MetaError> + ?Sized,
{
    for (ident, table_meta) in mv_table_metas_of_source(kv_api, tenant, source_table_id).await? {
        if let Some(updated) = invalidated(&table_meta.data, reason) {
            info!(
                "mark materialized view {} invalid: source table {} {}",
                ident.table_id,
                source_table_id,
                reason.as_str()
            );
            txn.condition.push(txn_cond_eq_seq(&ident, table_meta.seq));
            txn.if_then.push(txn_put_pb(&ident, &updated));
        }
    }
    Ok(())
}

/// Add to `txn` the operations that make the MVs of `source_table_id` valid again.
///
/// The caller has established that the source table is active after `txn`, under the name
/// `current_database_name.current_table_name`. An MV is restored when its reason is a
/// lifecycle one (see [`MVInvalidReason::restorable`]) and that name is the one it was
/// created against, recorded in its `TableMeta` options.
pub async fn restore_mvs_of_source<KV>(
    kv_api: &KV,
    txn: &mut TxnRequest,
    tenant: &Tenant,
    source_table_id: u64,
    current_database_name: &str,
    current_table_name: &str,
) -> Result<(), MetaError>
where
    KV: kvapi::KVApi<Error = MetaError> + ?Sized,
{
    for (ident, table_meta) in mv_table_metas_of_source(kv_api, tenant, source_table_id).await? {
        if bound_to(&table_meta.data, current_database_name, current_table_name)
            && let Some(updated) = restored(&table_meta.data)
        {
            info!(
                "restore materialized view {}: source table {} is {}.{} again",
                ident.table_id, source_table_id, current_database_name, current_table_name
            );
            txn.condition.push(txn_cond_eq_seq(&ident, table_meta.seq));
            txn.if_then.push(txn_put_pb(&ident, &updated));
        }
    }
    Ok(())
}

/// Add to `txn` the MV updates of renaming `source_table_id` to
/// `new_database_name.new_table_name`: an MV bound to that name is restored, every other MV is
/// marked `SourceRenamed`.
pub async fn rename_source_of_mvs<KV>(
    kv_api: &KV,
    txn: &mut TxnRequest,
    tenant: &Tenant,
    source_table_id: u64,
    new_database_name: &str,
    new_table_name: &str,
) -> Result<(), MetaError>
where
    KV: kvapi::KVApi<Error = MetaError> + ?Sized,
{
    for (ident, table_meta) in mv_table_metas_of_source(kv_api, tenant, source_table_id).await? {
        let updated = if bound_to(&table_meta.data, new_database_name, new_table_name) {
            restored(&table_meta.data)
        } else {
            invalidated(&table_meta.data, MVInvalidReason::SourceRenamed)
        };
        if let Some(updated) = updated {
            info!(
                "rename source table {} to {}.{}: materialized view {} is now {}",
                source_table_id,
                new_database_name,
                new_table_name,
                ident.table_id,
                updated
                    .materialized_view_invalid_reason()
                    .unwrap_or("valid")
            );
            txn.condition.push(txn_cond_eq_seq(&ident, table_meta.seq));
            txn.if_then.push(txn_put_pb(&ident, &updated));
        }
    }
    Ok(())
}

/// Add to `txn` the MV updates of dropping a database: every MV whose source is one of
/// `source_table_ids` (the tables of that database) is marked `reason`.
///
/// This walks all MVs of the tenant once rather than the edges of every table of the database:
/// MVs are few, tables may be many.
pub async fn mark_mvs_of_sources_invalid<KV>(
    kv_api: &KV,
    txn: &mut TxnRequest,
    tenant: &Tenant,
    source_table_ids: &BTreeSet<u64>,
    reason: MVInvalidReason,
) -> Result<(), MetaError>
where
    KV: kvapi::KVApi<Error = MetaError> + ?Sized,
{
    if source_table_ids.is_empty() {
        return Ok(());
    }
    for (ident, table_meta) in all_mv_table_metas(kv_api, tenant).await? {
        let Ok(source_table_id) = table_meta.data.materialized_view_source_table_id() else {
            continue;
        };
        if !source_table_ids.contains(&source_table_id) {
            continue;
        }
        if let Some(updated) = invalidated(&table_meta.data, reason) {
            info!(
                "mark materialized view {} invalid: source table {} {}",
                ident.table_id,
                source_table_id,
                reason.as_str()
            );
            txn.condition.push(txn_cond_eq_seq(&ident, table_meta.seq));
            txn.if_then.push(txn_put_pb(&ident, &updated));
        }
    }
    Ok(())
}

/// Add to `txn` the MV updates of undropping database `database_name`: `tables` are its active
/// tables after `txn`, by id and name. An MV is restored when it is bound to one of them under
/// that name.
pub async fn restore_mvs_of_sources<KV>(
    kv_api: &KV,
    txn: &mut TxnRequest,
    tenant: &Tenant,
    database_name: &str,
    tables: &HashMap<u64, String>,
) -> Result<(), MetaError>
where
    KV: kvapi::KVApi<Error = MetaError> + ?Sized,
{
    if tables.is_empty() {
        return Ok(());
    }
    for (ident, table_meta) in all_mv_table_metas(kv_api, tenant).await? {
        let Ok(source_table_id) = table_meta.data.materialized_view_source_table_id() else {
            continue;
        };
        let Some(table_name) = tables.get(&source_table_id) else {
            continue;
        };
        if bound_to(&table_meta.data, database_name, table_name)
            && let Some(updated) = restored(&table_meta.data)
        {
            info!(
                "restore materialized view {}: source table {} is {}.{} again",
                ident.table_id, source_table_id, database_name, table_name
            );
            txn.condition.push(txn_cond_eq_seq(&ident, table_meta.seq));
            txn.if_then.push(txn_put_pb(&ident, &updated));
        }
    }
    Ok(())
}

/// Add to `txn` the MV updates of renaming a database to `new_database_name`: `tables` are its
/// tables by id and name. An MV bound to one of them under the new database name is restored,
/// every other MV of those tables is marked `SourceDatabaseRenamed`.
pub async fn rename_database_of_mvs<KV>(
    kv_api: &KV,
    txn: &mut TxnRequest,
    tenant: &Tenant,
    new_database_name: &str,
    tables: &HashMap<u64, String>,
) -> Result<(), MetaError>
where
    KV: kvapi::KVApi<Error = MetaError> + ?Sized,
{
    if tables.is_empty() {
        return Ok(());
    }
    for (ident, table_meta) in all_mv_table_metas(kv_api, tenant).await? {
        let Ok(source_table_id) = table_meta.data.materialized_view_source_table_id() else {
            continue;
        };
        let Some(table_name) = tables.get(&source_table_id) else {
            continue;
        };
        let updated = if bound_to(&table_meta.data, new_database_name, table_name) {
            restored(&table_meta.data)
        } else {
            invalidated(&table_meta.data, MVInvalidReason::SourceDatabaseRenamed)
        };
        if let Some(updated) = updated {
            info!(
                "rename database of source table {} to {}: materialized view {} is now {}",
                source_table_id,
                new_database_name,
                ident.table_id,
                updated
                    .materialized_view_invalid_reason()
                    .unwrap_or("valid")
            );
            txn.condition.push(txn_cond_eq_seq(&ident, table_meta.seq));
            txn.if_then.push(txn_put_pb(&ident, &updated));
        }
    }
    Ok(())
}

/// The MV `TableMeta` with `reason` set, or `None` when nothing changes: it already holds
/// `reason`, or holds the final `SourceSchemaChanged`.
fn invalidated(table_meta: &TableMeta, reason: MVInvalidReason) -> Option<TableMeta> {
    let current = table_meta.materialized_view_invalid_reason();
    if current == Some(reason.as_str())
        || current == Some(MVInvalidReason::SourceSchemaChanged.as_str())
    {
        return None;
    }
    let mut updated = table_meta.clone();
    updated.options.insert(
        OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON.to_string(),
        reason.as_str().to_string(),
    );
    Some(updated)
}

/// The MV `TableMeta` made valid, or `None` when it is valid already or its reason is not
/// restorable.
fn restored(table_meta: &TableMeta) -> Option<TableMeta> {
    if !table_meta.materialized_view_invalid_reason_restorable() {
        return None;
    }
    let mut updated = table_meta.clone();
    updated
        .options
        .remove(OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON);
    Some(updated)
}

/// Whether the MV with `table_meta` was created against `database_name.table_name`.
fn bound_to(table_meta: &TableMeta, database_name: &str, table_name: &str) -> bool {
    let bound = |key: &str| table_meta.options.get(key).map(String::as_str);
    bound(OPT_KEY_MATERIALIZED_VIEW_SOURCE_DATABASE_NAME) == Some(database_name)
        && bound(OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_NAME) == Some(table_name)
}

/// The table metas of the MVs depending on `source_table_id`, by the dependency edges. A
/// dropped MV, or one without a table meta, is skipped.
async fn mv_table_metas_of_source<KV>(
    kv_api: &KV,
    tenant: &Tenant,
    source_table_id: u64,
) -> Result<Vec<(TableId, SeqV<TableMeta>)>, MetaError>
where
    KV: kvapi::KVApi<Error = MetaError> + ?Sized,
{
    let mv_ids = mv_ids_of_source(kv_api, tenant, source_table_id).await?;
    live_mv_table_metas(kv_api, mv_ids).await
}

/// The table metas of every MV of the tenant, by the definitions, dropped MVs excluded.
async fn all_mv_table_metas<KV>(
    kv_api: &KV,
    tenant: &Tenant,
) -> Result<Vec<(TableId, SeqV<TableMeta>)>, MetaError>
where
    KV: kvapi::KVApi<Error = MetaError> + ?Sized,
{
    let prefix = DirName::new(MVDefinitionIdent::new(tenant, 0));
    let mv_ids = kv_api
        .list_pb_vec(ListOptions::unlimited(&prefix))
        .await?
        .into_iter()
        .map(|(ident, _)| *ident.name())
        .collect::<Vec<_>>();
    live_mv_table_metas(kv_api, mv_ids).await
}

async fn live_mv_table_metas<KV>(
    kv_api: &KV,
    mv_ids: Vec<u64>,
) -> Result<Vec<(TableId, SeqV<TableMeta>)>, MetaError>
where
    KV: kvapi::KVApi<Error = MetaError> + ?Sized,
{
    if mv_ids.is_empty() {
        return Ok(vec![]);
    }
    let idents = mv_ids.into_iter().map(TableId::new).collect::<Vec<_>>();
    let table_metas = kv_api.get_pb_values_vec(idents.clone()).await?;
    Ok(idents
        .into_iter()
        .zip(table_metas)
        .filter_map(|(ident, table_meta)| match table_meta {
            Some(table_meta) if table_meta.data.drop_on.is_none() => Some((ident, table_meta)),
            Some(_) => None,
            None => {
                warn!(
                    "materialized view {} has a dependency edge but no table meta",
                    ident.table_id
                );
                None
            }
        })
        .collect())
}

async fn mv_ids_of_source<KV>(
    kv_api: &KV,
    tenant: &Tenant,
    source_table_id: u64,
) -> Result<Vec<u64>, MetaError>
where
    KV: kvapi::KVApi<Error = MetaError> + ?Sized,
{
    let source_mv_prefix = DirName::new(SourceTableMVIdent::new_generic(
        tenant,
        SourceTableMV::new(source_table_id, 0),
    ));
    Ok(kv_api
        .list_pb_vec(ListOptions::unlimited(&source_mv_prefix))
        .await?
        .into_iter()
        .map(|(ident, _)| ident.name().mv_table_id)
        .collect())
}

struct ListedMVTarget {
    mv: MVInfo,
    database_id: u64,
    database_name: String,
    name: String,
    source_table_id: u64,
}

type RawMVTarget = (
    u64,
    Option<SeqV<MVDefinition>>,
    Option<SeqV<TableMeta>>,
    Option<SeqV<DBIdTableName>>,
);

async fn mget_responses<KV>(
    kv_api: &KV,
    keys: &[String],
) -> Result<Vec<TxnGetResponse>, MetaError>
where
    KV: kvapi::KVApi<Error = MetaError> + ?Sized,
{
    if keys.is_empty() {
        return Ok(vec![]);
    }

    let mut values = Vec::with_capacity(keys.len());
    for chunk in keys.chunks(DEFAULT_MGET_SIZE) {
        values.extend(kv_api.mget_kv(chunk).await?);
    }
    if values.len() != keys.len() {
        return Err(invalid_reply(format!(
            "materialized-view MGET expected {} responses, got {}",
            keys.len(),
            values.len()
        ))
        .into());
    }

    Ok(keys
        .iter()
        .zip(values)
        .map(|(key, value)| TxnGetResponse::new(key, value.map(pb::SeqV::from)))
        .collect())
}

async fn list_database_mappings<KV>(
    kv_api: &KV,
    tenant: &Tenant,
) -> Result<HashMap<u64, String>, MetaError>
where
    KV: kvapi::KVApi<Error = MetaError> + ?Sized,
{
    let prefix = DirName::new(DatabaseNameIdent::new(tenant, "dummy"));
    Ok(kv_api
        .list_pb_vec(ListOptions::unlimited(&prefix))
        .await?
        .into_iter()
        .map(|(ident, database_id)| (database_id.data.db_id, ident.database_name().to_string()))
        .collect())
}

fn decode_database_responses(
    responses: Vec<TxnGetResponse>,
) -> Result<HashMap<u64, String>, MetaError> {
    let mut mappings = HashMap::with_capacity(responses.len());
    for response in responses {
        let (ident, database_id) = deserialize_struct_get_response::<DatabaseNameIdent>(response)?;
        if let Some(database_id) = database_id {
            mappings.insert(database_id.data.db_id, ident.database_name().to_string());
        }
    }
    Ok(mappings)
}

fn build_target(
    mv_id: u64,
    definition: Option<SeqV<MVDefinition>>,
    table_meta: Option<SeqV<TableMeta>>,
    table_name: Option<SeqV<DBIdTableName>>,
    database_mappings: &HashMap<u64, String>,
    filter: &MaterializedViewListFilter,
) -> Option<ListedMVTarget> {
    let definition = definition?;
    let (Some(table_meta), Some(table_name)) = (table_meta, table_name) else {
        warn!("materialized view {} has incomplete target metadata", mv_id);
        return None;
    };
    if table_meta.data.drop_on.is_some() || !is_materialized_view_engine(&table_meta.data.engine) {
        return None;
    }

    let Some(database_name) = database_mappings.get(&table_name.data.db_id) else {
        if filter.database_names.is_none() {
            warn!(
                "materialized view {} references inactive database {}",
                mv_id, table_name.data.db_id
            );
        }
        return None;
    };
    let source_table_id = match table_meta.data.materialized_view_source_table_id() {
        Ok(source_table_id) => source_table_id,
        Err(error) => {
            warn!(
                "materialized view {} has invalid source metadata: {}",
                mv_id, error
            );
            return None;
        }
    };

    if filter
        .names
        .as_ref()
        .is_some_and(|names| !names.contains(&table_name.data.table_name))
        || filter
            .source_table_ids
            .as_ref()
            .is_some_and(|ids| !ids.contains(&source_table_id))
    {
        return None;
    }

    Some(ListedMVTarget {
        mv: MVInfo {
            mv_id,
            definition,
            table_meta,
        },
        database_id: table_name.data.db_id,
        database_name: database_name.clone(),
        name: table_name.data.table_name,
        source_table_id,
    })
}

async fn list_materialized_views_impl<KV>(
    kv_api: &KV,
    tenant: &Tenant,
    filter: &MaterializedViewListFilter,
) -> Result<Vec<ListedMaterializedView>, MetaError>
where
    KV: kvapi::KVApi<Error = MetaError> + ?Sized,
{
    let exact_mv_ids = filter.materialized_view_ids.as_ref();
    if exact_mv_ids.is_some_and(BTreeSet::is_empty) {
        return Ok(vec![]);
    }

    let exact_database_names = filter.database_names.as_ref();
    if exact_database_names.is_some_and(BTreeSet::is_empty) {
        return Ok(vec![]);
    }

    let (raw_targets, database_mappings): (Vec<RawMVTarget>, HashMap<u64, String>) =
        if let Some(mv_ids) = exact_mv_ids {
            let mv_ids = mv_ids.iter().copied().collect::<Vec<_>>();
            let mut keys = Vec::with_capacity(
                mv_ids.len() * 3 + exact_database_names.map_or(0, BTreeSet::len),
            );
            keys.extend(
                mv_ids
                    .iter()
                    .map(|mv_id| MVDefinitionIdent::new(tenant, *mv_id).to_string_key()),
            );
            keys.extend(
                mv_ids
                    .iter()
                    .map(|mv_id| TableId::new(*mv_id).to_string_key()),
            );
            keys.extend(
                mv_ids
                    .iter()
                    .map(|mv_id| TableIdToName { table_id: *mv_id }.to_string_key()),
            );
            if let Some(database_names) = exact_database_names {
                keys.extend(
                    database_names
                        .iter()
                        .map(|name| DatabaseNameIdent::new(tenant, name).to_string_key()),
                );
            }

            let (mut responses, listed_database_mappings) = if exact_database_names.is_some() {
                (mget_responses(kv_api, &keys).await?, None)
            } else {
                let (responses, mappings) = futures::try_join!(
                    mget_responses(kv_api, &keys),
                    list_database_mappings(kv_api, tenant)
                )?;
                (responses, Some(mappings))
            };

            let database_responses = responses.split_off(mv_ids.len() * 3);
            let table_name_responses = responses.split_off(mv_ids.len() * 2);
            let table_meta_responses = responses.split_off(mv_ids.len());
            let definition_responses = responses;
            let database_mappings = match listed_database_mappings {
                Some(mappings) => mappings,
                None => decode_database_responses(database_responses)?,
            };

            let mut targets = Vec::with_capacity(mv_ids.len());
            for (mv_id, ((definition, table_meta), table_name)) in mv_ids.into_iter().zip(
                definition_responses
                    .into_iter()
                    .zip(table_meta_responses)
                    .zip(table_name_responses),
            ) {
                let (_, definition) =
                    deserialize_struct_get_response::<MVDefinitionIdent>(definition)?;
                let (_, table_meta) = deserialize_struct_get_response::<TableId>(table_meta)?;
                let (_, table_name) = deserialize_struct_get_response::<TableIdToName>(table_name)?;
                targets.push((mv_id, definition, table_meta, table_name));
            }
            (targets, database_mappings)
        } else {
            let prefix = DirName::new(MVDefinitionIdent::new(tenant, 0));
            let definitions = kv_api.list_pb_vec(ListOptions::unlimited(&prefix)).await?;
            if definitions.is_empty() {
                return Ok(vec![]);
            }

            let mv_ids = definitions
                .iter()
                .map(|(ident, _)| *ident.name())
                .collect::<Vec<_>>();
            let mut keys = Vec::with_capacity(
                mv_ids.len() * 2 + exact_database_names.map_or(0, BTreeSet::len),
            );
            keys.extend(
                mv_ids
                    .iter()
                    .map(|mv_id| TableId::new(*mv_id).to_string_key()),
            );
            keys.extend(
                mv_ids
                    .iter()
                    .map(|mv_id| TableIdToName { table_id: *mv_id }.to_string_key()),
            );
            if let Some(database_names) = exact_database_names {
                keys.extend(
                    database_names
                        .iter()
                        .map(|name| DatabaseNameIdent::new(tenant, name).to_string_key()),
                );
            }

            let (mut responses, listed_database_mappings) = if exact_database_names.is_some() {
                (mget_responses(kv_api, &keys).await?, None)
            } else {
                let (responses, mappings) = futures::try_join!(
                    mget_responses(kv_api, &keys),
                    list_database_mappings(kv_api, tenant)
                )?;
                (responses, Some(mappings))
            };

            let database_responses = responses.split_off(mv_ids.len() * 2);
            let table_name_responses = responses.split_off(mv_ids.len());
            let table_meta_responses = responses;
            let database_mappings = match listed_database_mappings {
                Some(mappings) => mappings,
                None => decode_database_responses(database_responses)?,
            };

            let mut targets = Vec::with_capacity(mv_ids.len());
            for (((ident, definition), table_meta), table_name) in definitions
                .into_iter()
                .zip(table_meta_responses)
                .zip(table_name_responses)
            {
                let (_, table_meta) = deserialize_struct_get_response::<TableId>(table_meta)?;
                let (_, table_name) = deserialize_struct_get_response::<TableIdToName>(table_name)?;
                targets.push((*ident.name(), Some(definition), table_meta, table_name));
            }
            (targets, database_mappings)
        };

    let targets = raw_targets
        .into_iter()
        .filter_map(|(mv_id, definition, table_meta, table_name)| {
            build_target(
                mv_id,
                definition,
                table_meta,
                table_name,
                &database_mappings,
                filter,
            )
        })
        .collect::<Vec<_>>();
    if targets.is_empty() {
        return Ok(vec![]);
    }

    let source_ids = targets
        .iter()
        .map(|target| target.source_table_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let mut source_keys = Vec::with_capacity(source_ids.len() * 2);
    source_keys.extend(source_ids.iter().map(|source_id| {
        TableIdToName {
            table_id: *source_id,
        }
        .to_string_key()
    }));
    source_keys.extend(
        source_ids
            .iter()
            .map(|source_id| TableId::new(*source_id).to_string_key()),
    );

    let mut responses = mget_responses(kv_api, &source_keys).await?;
    let source_meta_responses = responses.split_off(source_ids.len());
    let source_name_responses = responses;

    let mut source_facts = HashMap::with_capacity(source_ids.len());
    for (source_id, (table_name, table_meta)) in source_ids
        .into_iter()
        .zip(source_name_responses.into_iter().zip(source_meta_responses))
    {
        let (_, table_name) = deserialize_struct_get_response::<TableIdToName>(table_name)?;
        let (_, table_meta) = deserialize_struct_get_response::<TableId>(table_meta)?;
        let is_active = table_meta.is_some_and(|meta| meta.data.drop_on.is_none());
        source_facts.insert(source_id, (table_name, is_active));
    }

    let source_database_ids = source_facts
        .values()
        .filter_map(|(table_name, _)| table_name.as_ref().map(|name| name.data.db_id))
        .collect::<BTreeSet<_>>();
    let source_database_keys = source_database_ids
        .iter()
        .map(|database_id| DatabaseIdToName::new(*database_id).to_string_key())
        .chain(
            source_database_ids
                .iter()
                .map(|database_id| DatabaseId::new(*database_id).to_string_key()),
        )
        .collect::<Vec<_>>();
    let mut source_database_name_responses = mget_responses(kv_api, &source_database_keys).await?;
    let source_database_meta_responses =
        source_database_name_responses.split_off(source_database_ids.len());
    let mut source_database_names = HashMap::with_capacity(source_database_ids.len());
    // DROP DATABASE marks only DatabaseMeta and keeps both the id-to-name mapping and the tables'
    // TableMeta until GC, so a source in a dropped database must be checked through its database.
    let mut active_source_database_ids = BTreeSet::new();
    for (database_id, (name_response, meta_response)) in source_database_ids.into_iter().zip(
        source_database_name_responses
            .into_iter()
            .zip(source_database_meta_responses),
    ) {
        let (_, database_name) =
            deserialize_struct_get_response::<DatabaseIdToName>(name_response)?;
        if let Some(database_name) = database_name {
            source_database_names
                .insert(database_id, database_name.data.database_name().to_string());
        }
        let (_, database_meta) = deserialize_struct_get_response::<DatabaseId>(meta_response)?;
        if database_meta.is_some_and(|meta| meta.data.drop_on.is_none()) {
            active_source_database_ids.insert(database_id);
        }
    }

    let mut listed = Vec::with_capacity(targets.len());
    for target in targets {
        let (table_name, is_active) = source_facts
            .get(&target.source_table_id)
            .cloned()
            .unwrap_or((None, false));
        let (database_id, database_name, table_name, is_active, database_dropped) = match table_name
        {
            Some(table_name) => {
                let database_id = table_name.data.db_id;
                let database_dropped = !active_source_database_ids.contains(&database_id);
                (
                    Some(database_id),
                    source_database_names.get(&database_id).cloned(),
                    Some(table_name.data.table_name),
                    is_active && !database_dropped,
                    database_dropped,
                )
            }
            None => (None, None, None, is_active, false),
        };
        listed.push(ListedMaterializedView {
            mv: target.mv,
            database_id: target.database_id,
            database_name: target.database_name,
            name: target.name,
            source: ListedMVSource {
                table_id: target.source_table_id,
                database_id,
                database_name,
                table_name,
                is_active,
                database_dropped,
            },
        });
    }

    Ok(listed)
}

async fn list_mvs_by_source_table_id_impl<KV>(
    kv_api: &KV,
    tenant: &Tenant,
    source_table_id: u64,
) -> Result<Vec<MVInfo>, MetaError>
where
    KV: kvapi::KVApi<Error = MetaError> + ?Sized,
{
    let mv_ids = mv_ids_of_source(kv_api, tenant, source_table_id).await?;
    if mv_ids.is_empty() {
        return Ok(vec![]);
    }

    let mut keys = Vec::with_capacity(mv_ids.len() * 2);
    keys.extend(
        mv_ids
            .iter()
            .map(|mv_id| MVDefinitionIdent::new(tenant, *mv_id).to_string_key()),
    );
    keys.extend(
        mv_ids
            .iter()
            .map(|mv_id| TableId::new(*mv_id).to_string_key()),
    );

    let mut responses = mget_responses(kv_api, &keys).await?;
    let table_meta_responses = responses.split_off(mv_ids.len());
    let definition_responses = responses;
    let mut mvs = Vec::with_capacity(mv_ids.len());

    for (mv_id, (definition_response, table_meta_response)) in mv_ids
        .iter()
        .zip(definition_responses.into_iter().zip(table_meta_responses))
    {
        let (_, definition) =
            deserialize_struct_get_response::<MVDefinitionIdent>(definition_response)?;
        let (_, table_meta) = deserialize_struct_get_response::<TableId>(table_meta_response)?;

        let (Some(definition), Some(table_meta)) = (definition, table_meta) else {
            warn!(
                "source table {} references MV {} with incomplete metadata",
                source_table_id, mv_id
            );
            continue;
        };
        mvs.push(MVInfo {
            mv_id: *mv_id,
            definition,
            table_meta,
        });
    }

    Ok(mvs)
}

impl<KV> MaterializedViewApi for KV
where
    KV: Send + Sync,
    KV: kvapi::KVApi<Error = MetaError> + ?Sized,
{
}
