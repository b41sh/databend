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

use databend_common_expression::TableSchema;
use databend_meta_client::types::SeqV;

use super::TableMeta;
use crate::app_error::AppError;
use crate::app_error::InvalidMaterializedView;

mod mv_definition_ident;
mod source_table_mv_ident;

pub use mv_definition_ident::MVDefinitionIdent;
pub use mv_definition_ident::MVDefinitionResource;
pub use source_table_mv_ident::MVSourceBinding;
pub use source_table_mv_ident::SourceTableMV;
pub use source_table_mv_ident::SourceTableMVIdent;
pub use source_table_mv_ident::SourceTableMVResource;

pub const MATERIALIZED_VIEW_ENGINE: &str = "MATERIALIZED_VIEW";
/// Internal table option containing the source table ID of a materialized view.
pub const OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_ID: &str = "materialized_view_source_table_id";
/// Internal table option containing the source table sequence captured when the MV was created
/// or last refreshed.
pub const OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_SEQ: &str = "materialized_view_source_table_seq";
/// Internal table options recording the database and table name the source had when the MV was
/// created. A renamed source is restored to validity when it gets this name back.
pub const OPT_KEY_MATERIALIZED_VIEW_SOURCE_DATABASE_NAME: &str =
    "materialized_view_source_database_name";
pub const OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_NAME: &str = "materialized_view_source_table_name";
/// Internal table option holding why the MV is invalid; absent while it is valid. Source DDL that
/// breaks the binding writes it in its own transaction, see [`MVInvalidReason`].
pub const OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON: &str = "materialized_view_invalid_reason";
/// Hidden physical column used to match source DELETE/UPDATE rows during refresh.
pub const MATERIALIZED_VIEW_SOURCE_ROW_ID_COLUMN: &str = "_mv_source_row_id";

pub fn is_materialized_view_engine(engine: &str) -> bool {
    engine == MATERIALIZED_VIEW_ENGINE
}

pub fn invalidates_mv_source_bindings(old_meta: &TableMeta, new_meta: &TableMeta) -> bool {
    if old_meta.schema == new_meta.schema {
        return false;
    }

    // Match each old column by ID, then compare the complete TableField. A missing or changed
    // old field invalidates the binding; fields that exist only in the new schema (ADD COLUMN)
    // are intentionally ignored.
    old_meta.schema.fields().iter().any(|old_field| {
        new_meta
            .schema
            .fields()
            .iter()
            .find(|new_field| new_field.column_id == old_field.column_id)
            != Some(old_field)
    })
}

impl TableMeta {
    /// Return the source table ID required by a materialized view.
    pub fn materialized_view_source_table_id(&self) -> Result<u64, AppError> {
        let source_table_id = self
            .options
            .get(OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_ID)
            .ok_or_else(|| {
                AppError::InvalidMaterializedView(InvalidMaterializedView::new(format!(
                    "missing required table option {OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_ID}"
                )))
            })?;

        source_table_id.parse::<u64>().map_err(|_| {
            AppError::InvalidMaterializedView(InvalidMaterializedView::new(format!(
                "invalid table option {OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_ID}: '{source_table_id}'"
            )))
        })
    }

    /// Return the source table sequence recorded by a materialized view.
    pub fn materialized_view_source_table_seq(&self) -> Result<u64, AppError> {
        let source_table_seq = self
            .options
            .get(OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_SEQ)
            .ok_or_else(|| {
                AppError::InvalidMaterializedView(InvalidMaterializedView::new(format!(
                    "missing required table option {OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_SEQ}"
                )))
            })?;

        source_table_seq.parse::<u64>().map_err(|_| {
            AppError::InvalidMaterializedView(InvalidMaterializedView::new(format!(
                "invalid table option {OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_SEQ}: '{source_table_seq}'"
            )))
        })
    }
}

/// Definition associated with a materialized-view table.
///
/// A materialized view reuses table metadata and storage for its materialized
/// data, and its table ID is also its materialized view ID. [`TableMeta`]
/// describes how the data is stored and whether the view is valid
/// (`OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON`), while this record stores the
/// defining query and the columns returned to users under the same table ID.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MVDefinition {
    pub original_query: String,
    pub query: String,
    pub logical_schema: TableSchema,
    pub sync_creation: bool,
}

impl TableMeta {
    /// Why the materialized view is invalid; `None` while it is valid.
    ///
    /// Validity is explicit: source DDL that breaks the binding (DROP TABLE, DROP DATABASE,
    /// RENAME TABLE, RENAME DATABASE, invalidating schema changes) writes the reason into every
    /// dependent MV's `TableMeta` in its own transaction. Readers need no other record to
    /// decide whether the MV is valid. Lifecycle reasons are cleared again when the source is
    /// restored to the identity the MV was bound to (UNDROP, RENAME back); a schema change is
    /// final.
    pub fn materialized_view_invalid_reason(&self) -> Option<&str> {
        self.options
            .get(OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON)
            .map(String::as_str)
    }

    pub fn is_valid_materialized_view(&self) -> bool {
        self.materialized_view_invalid_reason().is_none()
    }

    /// Whether a restore may clear the current reason once the source identity matches again.
    /// A missing reason needs no restore; an unknown reason is kept.
    pub fn materialized_view_invalid_reason_restorable(&self) -> bool {
        self.materialized_view_invalid_reason()
            .and_then(MVInvalidReason::parse)
            .is_some_and(|reason| reason.restorable())
    }
}

/// Why a materialized view is invalid, as stored under
/// [`OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MVInvalidReason {
    /// The source table was dropped. Cleared by UNDROP TABLE.
    SourceDropped,
    /// The database holding the source table was dropped. Cleared by UNDROP DATABASE.
    SourceDatabaseDropped,
    /// The source table was renamed or moved to another database. Cleared by renaming it back
    /// to the database and name the MV was created against.
    SourceRenamed,
    /// The database holding the source table was renamed. Cleared by renaming it back.
    SourceDatabaseRenamed,
    /// A column the MV may depend on was renamed, dropped, or changed. Never cleared.
    SourceSchemaChanged,
}

impl MVInvalidReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceDropped => "source table was dropped",
            Self::SourceDatabaseDropped => "source database was dropped",
            Self::SourceRenamed => "source table was renamed",
            Self::SourceDatabaseRenamed => "source database was renamed",
            Self::SourceSchemaChanged => "source table schema was changed",
        }
    }

    pub fn parse(reason: &str) -> Option<Self> {
        [
            Self::SourceDropped,
            Self::SourceDatabaseDropped,
            Self::SourceRenamed,
            Self::SourceDatabaseRenamed,
            Self::SourceSchemaChanged,
        ]
        .into_iter()
        .find(|candidate| candidate.as_str() == reason)
    }

    /// Whether the source can come back to the identity the MV was bound to.
    pub const fn restorable(self) -> bool {
        !matches!(self, Self::SourceSchemaChanged)
    }
}

/// Materialized-view metadata supplied only while creating a table.
///
/// `definition` is persisted as [`MVDefinition`]. `create_table` reads the
/// current source `TableMeta` itself to reject a missing or dropped source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateMaterializedViewMeta {
    pub definition: MVDefinition,
}

/// Complete metadata needed to use one materialized view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MVInfo {
    pub mv_id: u64,
    pub definition: SeqV<MVDefinition>,
    pub table_meta: SeqV<TableMeta>,
}

/// Safe equality constraints accepted by the materialized-view listing API.
///
/// `None` means that the caller did not extract a necessary constraint for that field. The API
/// applies these constraints only to prune candidates; SQL callers must still evaluate their
/// complete filter expression.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MaterializedViewListFilter {
    pub materialized_view_ids: Option<BTreeSet<u64>>,
    pub database_names: Option<BTreeSet<String>>,
    pub names: Option<BTreeSet<String>>,
    pub source_table_ids: Option<BTreeSet<u64>>,
}

/// Source metadata collected for one listed materialized view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListedMVSource {
    pub table_id: u64,
    /// Current database identity resolved from the source table's ID-to-name mapping.
    pub database_id: Option<u64>,
    pub database_name: Option<String>,
    pub table_name: Option<String>,
    /// Whether source `TableMeta` exists and is not soft-dropped, and its database is not dropped.
    pub is_active: bool,
    /// Whether the source's current database is dropped. `is_active` is false in that case too;
    /// this flag only tells management paths which of the two was dropped.
    pub database_dropped: bool,
}

/// Complete metadata needed by management and system-table listing paths.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListedMaterializedView {
    pub mv: MVInfo,
    pub database_id: u64,
    pub database_name: String,
    pub name: String,
    pub source: ListedMVSource,
}

/// The valid MVs of one source table, for query rewrite.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MVSourceBindingSnapshot {
    pub materialized_views: Vec<MVInfo>,
}

#[cfg(test)]
mod tests {
    use super::MVInvalidReason;
    use super::OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON;
    use super::OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_ID;
    use super::OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_SEQ;
    use crate::schema::TableMeta;

    #[test]
    fn test_mv_invalid_reason_round_trip_and_restorable() {
        for reason in [
            MVInvalidReason::SourceDropped,
            MVInvalidReason::SourceDatabaseDropped,
            MVInvalidReason::SourceRenamed,
            MVInvalidReason::SourceDatabaseRenamed,
            MVInvalidReason::SourceSchemaChanged,
        ] {
            assert_eq!(MVInvalidReason::parse(reason.as_str()), Some(reason));
            assert_eq!(
                reason.restorable(),
                reason != MVInvalidReason::SourceSchemaChanged
            );
        }
        assert_eq!(MVInvalidReason::parse("unknown"), None);

        let mut table_meta = TableMeta::default();
        assert!(table_meta.is_valid_materialized_view());
        assert!(!table_meta.materialized_view_invalid_reason_restorable());
        table_meta.options.insert(
            OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON.to_string(),
            MVInvalidReason::SourceRenamed.as_str().to_string(),
        );
        assert!(!table_meta.is_valid_materialized_view());
        assert!(table_meta.materialized_view_invalid_reason_restorable());
        table_meta.options.insert(
            OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON.to_string(),
            "unknown".to_string(),
        );
        assert!(!table_meta.materialized_view_invalid_reason_restorable());
    }

    #[test]
    fn test_materialized_view_source_table_id() {
        let mut table_meta = TableMeta::default();

        assert!(table_meta.materialized_view_source_table_id().is_err());

        table_meta.options.insert(
            OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_ID.to_string(),
            "invalid".to_string(),
        );
        assert!(table_meta.materialized_view_source_table_id().is_err());

        table_meta.options.insert(
            OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_ID.to_string(),
            "42".to_string(),
        );
        assert_eq!(table_meta.materialized_view_source_table_id().unwrap(), 42);
    }

    #[test]
    fn test_materialized_view_source_table_seq() {
        let mut table_meta = TableMeta::default();

        assert!(table_meta.materialized_view_source_table_seq().is_err());

        table_meta.options.insert(
            OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_SEQ.to_string(),
            "invalid".to_string(),
        );
        assert!(table_meta.materialized_view_source_table_seq().is_err());

        table_meta.options.insert(
            OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_SEQ.to_string(),
            "42".to_string(),
        );
        assert_eq!(table_meta.materialized_view_source_table_seq().unwrap(), 42);
    }
}
