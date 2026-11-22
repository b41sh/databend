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

use databend_common_expression::ComputedExpr;
use databend_common_expression::TableField;
use databend_common_expression::TableSchema;
use databend_meta_client::types::SeqV;

use super::TableMeta;
use crate::app_error::AppError;
use crate::app_error::InvalidMaterializedView;

mod mv_definition_ident;
mod mv_source_binding_version_ident;
mod source_table_mv_ident;

pub use mv_definition_ident::MVDefinitionIdent;
pub use mv_definition_ident::MVDefinitionResource;
pub use mv_source_binding_version_ident::LegacyMVSourceBindingVersionIdent;
pub use mv_source_binding_version_ident::LegacyMVSourceBindingVersionResource;
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
/// created, the names its definition reads the source by.
pub const OPT_KEY_MATERIALIZED_VIEW_SOURCE_DATABASE_NAME: &str =
    "materialized_view_source_database_name";
pub const OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_NAME: &str = "materialized_view_source_table_name";
/// Internal table option holding why the MV is invalid; absent while it is valid. It is written
/// on a best-effort basis after the source DDL that breaks the binding, see [`MVInvalidReason`].
pub const OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON: &str = "materialized_view_invalid_reason";
/// Internal table option recording, as JSON, the source columns the MV reads, see
/// [`MVSourceColumn`]. Written by CREATE and never changed after; an MV created by an older
/// version gets all the source columns at the first schema change of its source.
pub const OPT_KEY_MATERIALIZED_VIEW_SOURCE_COLUMNS: &str = "materialized_view_source_columns";
/// Hidden physical column used to match source DELETE/UPDATE rows during refresh.
pub const MATERIALIZED_VIEW_SOURCE_ROW_ID_COLUMN: &str = "_mv_source_row_id";

pub fn is_materialized_view_engine(engine: &str) -> bool {
    engine == MATERIALIZED_VIEW_ENGINE
}

/// One source column a materialized view reads, as it was when the MV was created.
///
/// The MV query names source columns in SQL, and its storage holds values of their types, so a
/// column the MV reads must keep its id, name, type, and computed expression. Defaults are not
/// recorded: they do not change the values of existing rows. Columns the MV does not read are not
/// recorded, so changing them keeps the MV valid.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MVSourceColumn {
    pub id: u32,
    pub name: String,
    /// `TableDataType::sql_name_explicit_null`, so a change of nullability is a change of type.
    #[serde(rename = "type")]
    pub data_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub computed: Option<String>,
}

impl MVSourceColumn {
    pub fn from_field(field: &TableField) -> Self {
        MVSourceColumn {
            id: field.column_id,
            name: field.name.clone(),
            data_type: field.data_type.sql_name_explicit_null(),
            computed: field.computed_expr.as_ref().map(|expr| match expr {
                ComputedExpr::Virtual(expr) => format!("VIRTUAL {expr}"),
                ComputedExpr::Stored(expr) => format!("STORED {expr}"),
            }),
        }
    }

    /// All the columns of `schema`, recorded for an MV of an older version, whose read columns
    /// are not known.
    pub fn from_schema(schema: &TableSchema) -> Vec<Self> {
        schema.fields().iter().map(Self::from_field).collect()
    }

    /// The table option value holding `columns`.
    pub fn encode(columns: &[Self]) -> String {
        serde_json::to_string(columns).expect("serializing MV source columns cannot fail")
    }

    /// Whether `schema` dropped, renamed, retyped, or recomputed one of `recorded`. Columns that
    /// are not recorded, such as those added by ADD COLUMN, are ignored.
    pub fn changed(recorded: &[Self], schema: &TableSchema) -> bool {
        recorded.iter().any(|column| {
            schema
                .fields()
                .iter()
                .find(|field| field.column_id == column.id)
                .is_none_or(|field| &Self::from_field(field) != column)
        })
    }
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

    /// The source columns recorded by a materialized view; `None` for an MV created by a version
    /// that did not record them.
    pub fn materialized_view_source_columns(
        &self,
    ) -> Result<Option<Vec<MVSourceColumn>>, AppError> {
        let Some(columns) = self.options.get(OPT_KEY_MATERIALIZED_VIEW_SOURCE_COLUMNS) else {
            return Ok(None);
        };
        serde_json::from_str(columns).map(Some).map_err(|error| {
            AppError::InvalidMaterializedView(InvalidMaterializedView::new(format!(
                "invalid table option {OPT_KEY_MATERIALIZED_VIEW_SOURCE_COLUMNS}: {error}"
            )))
        })
    }

    /// Whether the source schema `schema` changed a column this materialized view was created
    /// against. An MV without recorded columns cannot tell, and reports no change.
    pub fn materialized_view_source_schema_changed(
        &self,
        schema: &TableSchema,
    ) -> Result<bool, AppError> {
        Ok(self
            .materialized_view_source_columns()?
            .is_some_and(|recorded| MVSourceColumn::changed(&recorded, schema)))
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
    /// The reason is written on a best-effort basis after a source DDL that breaks the MV, see
    /// [`MVInvalidReason`]. It is for display and a quick rejection: readers still bind the
    /// definition by the source name and check the source table id, and check
    /// [`Self::materialized_view_source_schema_changed`], because a failed write leaves the
    /// reason out. A rename of the source table or of its database, and a drop of its database,
    /// write no reason: the definition no longer binds, and binds again once the source is back.
    pub fn materialized_view_invalid_reason(&self) -> Option<&str> {
        self.options
            .get(OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON)
            .map(String::as_str)
    }

    pub fn is_valid_materialized_view(&self) -> bool {
        self.materialized_view_invalid_reason().is_none()
    }

    /// Whether UNDROP TABLE of the source may clear the current reason. A missing reason needs no
    /// restore; an unknown reason is kept.
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
    /// The source table was dropped, or replaced by CREATE OR REPLACE TABLE, which drops it.
    /// Cleared by UNDROP TABLE of it, as in Snowflake.
    SourceDropped,
    /// The source table was swapped with another by ALTER TABLE ... SWAP WITH. Never cleared.
    SourceSwapped,
    /// A column the MV reads was renamed, dropped, or changed. Never cleared.
    SourceSchemaChanged,
}

impl MVInvalidReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceDropped => "source table was dropped",
            Self::SourceSwapped => "source table was swapped",
            Self::SourceSchemaChanged => "source table schema was changed",
        }
    }

    pub fn parse(reason: &str) -> Option<Self> {
        [
            Self::SourceDropped,
            Self::SourceSwapped,
            Self::SourceSchemaChanged,
        ]
        .into_iter()
        .find(|candidate| candidate.as_str() == reason)
    }

    /// Whether the source can come back as it was when the MV was created: only a dropped or
    /// replaced table, by UNDROP TABLE.
    pub const fn restorable(self) -> bool {
        matches!(self, Self::SourceDropped)
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

#[cfg(test)]
mod tests {
    use databend_common_expression::ComputedExpr;
    use databend_common_expression::TableDataType;
    use databend_common_expression::TableField;
    use databend_common_expression::TableSchema;
    use databend_common_expression::types::NumberDataType;

    use super::MVInvalidReason;
    use super::MVSourceColumn;
    use super::OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON;
    use super::OPT_KEY_MATERIALIZED_VIEW_SOURCE_COLUMNS;
    use super::OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_ID;
    use super::OPT_KEY_MATERIALIZED_VIEW_SOURCE_TABLE_SEQ;
    use crate::schema::TableMeta;

    #[test]
    fn test_mv_invalid_reason_round_trip_and_restorable() {
        for reason in [
            MVInvalidReason::SourceDropped,
            MVInvalidReason::SourceSwapped,
            MVInvalidReason::SourceSchemaChanged,
        ] {
            assert_eq!(MVInvalidReason::parse(reason.as_str()), Some(reason));
            assert_eq!(
                reason.restorable(),
                reason == MVInvalidReason::SourceDropped
            );
        }
        assert_eq!(MVInvalidReason::parse("unknown"), None);

        let mut table_meta = TableMeta::default();
        assert!(table_meta.is_valid_materialized_view());
        assert!(!table_meta.materialized_view_invalid_reason_restorable());
        table_meta.options.insert(
            OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON.to_string(),
            MVInvalidReason::SourceDropped.as_str().to_string(),
        );
        assert!(!table_meta.is_valid_materialized_view());
        assert!(table_meta.materialized_view_invalid_reason_restorable());
        table_meta.options.insert(
            OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON.to_string(),
            MVInvalidReason::SourceSwapped.as_str().to_string(),
        );
        assert!(!table_meta.materialized_view_invalid_reason_restorable());
        table_meta.options.insert(
            OPT_KEY_MATERIALIZED_VIEW_INVALID_REASON.to_string(),
            "unknown".to_string(),
        );
        assert!(!table_meta.materialized_view_invalid_reason_restorable());
    }

    /// A recorded column that is dropped, renamed, retyped, made nullable, or recomputed is a
    /// change; adding a column is not.
    #[test]
    fn test_mv_source_columns_changed() {
        let int = TableDataType::Number(NumberDataType::Int32);
        let schema = TableSchema::new(vec![
            TableField::new("a", int.clone()),
            TableField::new("b", TableDataType::String),
        ]);
        let recorded = MVSourceColumn::from_schema(&schema);
        assert!(!MVSourceColumn::changed(&recorded, &schema));

        let mut table_meta = TableMeta::default();
        assert_eq!(table_meta.materialized_view_source_columns().unwrap(), None);
        assert!(
            !table_meta
                .materialized_view_source_schema_changed(&schema)
                .unwrap()
        );
        table_meta.options.insert(
            OPT_KEY_MATERIALIZED_VIEW_SOURCE_COLUMNS.to_string(),
            MVSourceColumn::encode(&recorded),
        );
        assert_eq!(
            table_meta.materialized_view_source_columns().unwrap(),
            Some(recorded.clone())
        );

        let mut added = schema.clone();
        added
            .add_column(&TableField::new("c", int.clone()), 2)
            .unwrap();
        assert!(!MVSourceColumn::changed(&recorded, &added));

        let mut renamed = schema.clone();
        renamed.rename_field(0, "a2");
        assert!(MVSourceColumn::changed(&recorded, &renamed));

        let mut dropped = schema.clone();
        dropped.drop_column("b").unwrap();
        assert!(MVSourceColumn::changed(&recorded, &dropped));

        let mut retyped = schema.clone();
        retyped.fields[0].data_type = TableDataType::Number(NumberDataType::Int64);
        assert!(MVSourceColumn::changed(&recorded, &retyped));

        let mut nullable = schema.clone();
        nullable.fields[0].data_type = TableDataType::Nullable(Box::new(int));
        assert!(MVSourceColumn::changed(&recorded, &nullable));

        let mut computed = schema.clone();
        computed.fields[1].computed_expr = Some(ComputedExpr::Stored("'x'".to_string()));
        assert!(MVSourceColumn::changed(&recorded, &computed));

        table_meta.options.insert(
            OPT_KEY_MATERIALIZED_VIEW_SOURCE_COLUMNS.to_string(),
            "not json".to_string(),
        );
        assert!(table_meta.materialized_view_source_columns().is_err());
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
