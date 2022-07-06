<img src="https://repository-images.githubusercontent.com/302827809/a01c8064-0196-45d9-b326-1762d6d3062b" alt="databend" />
<div align="center">
 
<h4 align="center">
  <a href="https://databend.rs/doc/deploy/databend-cloud">Databend Cloud (beta)</a>  |
  <a href="https://databend.rs/doc">Documentation</a>  |
  <a href="https://perf.databend.rs">Benchmarking</a>  |
  <a href="https://github.com/datafuselabs/databend/issues/4591">Roadmap (v0.8)</a>

</h4>

<div>
<a href="https://link.databend.rs/join-slack">
<img src="https://badgen.net/badge/Slack/Join%20Databend/0abd59?icon=slack" alt="slack" />
</a>

<a href="https://github.com/datafuselabs/databend/actions">
<img src="https://img.shields.io/github/workflow/status/datafuselabs/databend/Release" alt="CI Status" />
</a>

<img src="https://img.shields.io/badge/Platform-Linux%2C%20macOS%2C%20ARM-green.svg?style=flat" alt="Linux Platform" />

<a href="https://opensource.org/licenses/Apache-2.0">
<img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="license" />
</a>

</div>
</div>
<br>

- [What is Databend?](#what-is-databend)
- [Architecture](#architecture)
- [Try Databend](#try-databend)
- [Getting Started](#getting-started)
- [Contributing](#contributing)
- [Community](#community)
- [Roadmap](#roadmap)

## What is Databend?

Databend is an open-source **Elastic** and **Workload-Aware** modern cloud data warehouse.

Databend uses the latest techniques in vectorized query processing to allow you to do blazing-fast data analytics on object storage([S3](https://aws.amazon.com/s3/), [Azure Blob](https://azure.microsoft.com/en-us/services/storage/blobs/) or [MinIO](https://min.io)).

- __Instant Elasticity__

  Databend completely separates storage from compute, which allows you easily scale up or scale down based on your application's needs.

- __Blazing Performance__

  Databend leverages data-level parallelism(Vectorized Query Execution) and instruction-level parallelism(SIMD) technology, offering blazing performance data analytics.
  
  
- __Git-like Storage__

  Databend stores data with snapshots. It's easy to query, clone, and restore historical data in tables.

- __Support for Semi-Structured Data__

  Databend supports [ingestion of semi-structured data](https://databend.rs/doc/load-data) in various formats like CSV, JSON, and Parquet, which are located in the cloud or your local file system; Databend also supports semi-structured data types: [ARRAY, MAP, JSON](https://databend.rs/doc/reference/data-types/data-type-semi-structured-types), which is easy to import and operate on semi-structured.

- __MySQL/ClickHouse Compatible__

  Databend is ANSI SQL compliant and MySQL/ClickHouse wire protocol compatible, making it easy to connect with existing tools([MySQL Client](https://databend.rs/doc/reference/api/mysql-handler), [ClickHouse Client](https://databend.rs/doc/reference/api/clickhouse-handler), [Vector](https://vector.dev/), [DBeaver](https://dbeaver.com/), [Jupyter](https://databend.rs/doc/integrations/gui-tool/jupyter), [JDBC](https://databend.rs/doc/develop), etc.).

- __Easy to Use__

  Databend has no indexes to build, no manual tuning required, no manual figuring out partitions or shard data, it’s all done for you as data is loaded into the table.
 
## Architecture

![Databend Architecture](https://datafuse-1253727613.cos.ap-hongkong.myqcloud.com/arch/datafuse-arch-20210817.svg)

## Try Databend

### Install Databend

Prepare the image (once) from Docker Hub (this will download about 170 MB data):

```shell
docker pull datafuselabs/databend
```

To run Databend quickly:
```shell
docker run --net=host  datafuselabs/databend
```

### Connect to Databend

[MySQL](https://databend.rs/doc/reference/api/mysql-handler) wire protocol on port `3307` 
```shell
mysql -h127.0.0.1 -uroot -P3307
```

Let's run some [benchmark queries](https://databend.rs/doc/performance/local-vector-performance).

## Getting Started

### Deployment

- [How to Deploy Databend With MinIO](https://databend.rs/doc/deploy/minio)
- [How to Deploy Databend With AWS S3](https://databend.rs/doc/deploy/s3)
- [How to Deploy Databend With Azure Blob Storage](https://databend.rs/doc/deploy/azure)
- [How to Deploy Databend With Wasabi Object Storage](https://databend.rs/doc/deploy/wasabi)
- [How to Deploy Databend With Scaleway OS](https://databend.rs/doc/deploy/scw)
- [How to Deploy Databend With Tencent COS](https://databend.rs/doc/deploy/cos)
- [How to Deploy Databend With Alibaba OSS](https://databend.rs/doc/deploy/oss)
- [How to Deploy Databend With QingCloud QingStore](https://databend.rs/doc/deploy/qingstore)
- [How to Deploy a Databend Local Cluster With MinIO](https://databend.rs/doc/deploy/local)
- [How to Deploy a Databend K8s Cluster With MinIO](https://databend.rs/doc/deploy/cluster-minio)
- [Databend Cloud (beta)](https://databend.rs/doc/deploy/databend-cloud)
 
### Connect

- [How to Connect Databend With MySQL Client](https://databend.rs/doc/reference/api/mysql-handler)
- [How to Connect Databend With ClickHouse Client](https://databend.rs/doc/reference/api/clickhouse-handler)
- [How to Connect Databend With DBeaver SQL IDE](https://databend.rs/doc/integrations/gui-tool/dbeaver)
- [How to Execute Queries in Python](https://databend.rs/doc/develop/python)
- [How to Query Databend in Jupyter Notebooks](https://databend.rs/doc/integrations/gui-tool/jupyter)
- [How to Execute Queries in Golang](https://databend.rs/doc/develop/golang)
- [How to Work With Databend in Node.js](https://databend.rs/doc/develop/nodejs)


### Users

- [How to Create a User](https://databend.rs/doc/reference/sql/ddl/user/user-create-user)
- [How to Grant Privileges to a User](https://databend.rs/doc/reference/sql/ddl/user/grant-privileges)
- [How to Revoke Privileges From a User](https://databend.rs/doc/reference/sql/ddl/user/revoke-privileges)
- [How to Create a Role](https://databend.rs/doc/reference/sql/ddl/user/user-create-role)
- [How to Grant Privileges to a Role](https://databend.rs/doc/reference/sql/ddl/user/grant-privileges)
- [How to Grant Role to a User](https://databend.rs/doc/reference/sql/ddl/user/grant-role)
- [How to Revoke Role From a User](https://databend.rs/doc/reference/sql/ddl/user/revoke-role)
 
### Tables

- [How to Create a Database](https://databend.rs/doc/reference/sql/ddl/database/ddl-create-database)
- [How to Drop a Database](https://databend.rs/doc/reference/sql/ddl/database/ddl-drop-database)
- [How to Create a Table](https://databend.rs/doc/reference/sql/ddl/table/ddl-create-table)
- [How to Drop a Table](https://databend.rs/doc/reference/sql/ddl/table/ddl-drop-table)
- [How to Rename a Table](https://databend.rs/doc/reference/sql/ddl/table/ddl-rename-table)
- [How to Truncate a Table](https://databend.rs/doc/reference/sql/ddl/table/ddl-truncate-table)

### Views

- [How to Create a View](https://databend.rs/doc/reference/sql/ddl/view/ddl-create-view)
- [How to Drop a View](https://databend.rs/doc/reference/sql/ddl/view/ddl-drop-view)
- [How to Alter a View](https://databend.rs/doc/reference/sql/ddl/view/ddl-alter-view)

## User-Defined Functions

- [How to Create a User-Defined Function](http://databend.rs/doc/reference/sql/ddl/udf/ddl-create-function)
- [How to Drop a User-Defined Function](http://databend.rs/doc/reference/sql/ddl/udf/ddl-drop-function)
- [How to Alter a User-Defined Function](http://databend.rs/doc/reference/sql/ddl/udf/ddl-alter-function)

 
### Load Data

- [How to Load Data From Local File System](https://databend.rs/doc/load-data/local)
- [How to Load Data From Amazon S3](https://databend.rs/doc/load-data/s3)
- [How to Load Data From Databend Stages](https://databend.rs/doc/load-data/stage)
- [How to Load Data From MySQL](https://databend.rs/doc/load-data/mysql)

### Use Case

- [Analyzing Github Repository With Databend](https://databend.rs/doc/learn/analyze-github-repo-with-databend)
- [Analyzing Nginx Access Logs With Databend](https://databend.rs/doc/learn/analyze-nginx-logs-with-databend-and-vector)
- [User Retention Analysis With Databend](https://databend.rs/doc/learn/analyze-user-retention-with-databend)
- [Conversion Funnel Analysis With Databend](https://databend.rs/doc/learn/analyze-funnel-with-databend)

### Performance

- [How to Benchmark Databend](https://databend.rs/doc/learn/analyze-ontime-with-databend-on-ec2-and-s3)


## Contributing

Databend is an open source project, you can help with ideas, code, or documentation, we appreciate any efforts that help us to make the project better!
Once the code been merged, your name will be stored in the **system.contributors** table forever.

To get started with contributing:

- [Building Databend From Source](https://databend.rs/doc/contributing/building-from-source)
- [The First Good Pull Request](https://databend.rs/doc/contributing/good-pr)


## Community

For general help in using Databend, please refer to the official documentation. For additional help, you can use one of these channels to ask a question:

- [Slack](https://link.databend.rs/join-slack) (For live discussion with the Community)
- [Github](https://github.com/datafuselabs/databend) (Feature/Bug reports, Contributions)
- [Twitter](https://twitter.com/Datafuse_Labs) (Get the news fast)
- [Weekly](https://weekly.databend.rs/) (A weekly newsletter about Databend)
- [I'm feeling lucky](https://link.databend.rs/i-m-feeling-lucky) (Pick up a good first issue now!)

## Roadmap
- [Roadmap v0.8](https://github.com/datafuselabs/databend/issues/4591)
- [Roadmap 2022](https://github.com/datafuselabs/databend/issues/3706)

## License

Databend is licensed under [Apache 2.0](LICENSE).

## Acknowledgement

- Databend is inspired by [ClickHouse](https://github.com/clickhouse/clickhouse) and [Snowflake](https://docs.snowflake.com/en/user-guide/intro-key-concepts.html#snowflake-architecture), its computing model is based on [apache-arrow](https://arrow.apache.org/).
- The [documentation website](https://databend.rs) hosted by [Vercel](https://vercel.com/?utm_source=databend&utm_campaign=oss).
- Thanks to [Mergify](https://mergify.com/) for sponsoring advanced features like Batch Merge.
- Thanks to [QingCloud](https://qingcloud.com) for sponsoring CI resources.









file_meta=FileMetaData {
    version: 2,
    schema: [
        SchemaElement {
            type_: None,
            type_length: None,
            repetition_type: None,
            name: "root",
            num_children: Some(
                2,
            ),
            converted_type: None,
            scale: None,
            precision: None,
            field_id: None,
            logical_type: None,
        },
        SchemaElement {
            type_: Some(
                Type(
                    1,
                ),
            ),
            type_length: None,
            repetition_type: Some(
                FieldRepetitionType(
                    0,
                ),
            ),
            name: "id",
            num_children: None,
            converted_type: None,
            scale: None,
            precision: None,
            field_id: None,
            logical_type: None,
        },
        SchemaElement {
            type_: None,
            type_length: None,
            repetition_type: Some(
                FieldRepetitionType(
                    0,
                ),
            ),
            name: "v",
            num_children: Some(
                2,
            ),
            converted_type: None,
            scale: None,
            precision: None,
            field_id: None,
            logical_type: None,
        },
        SchemaElement {
            type_: Some(
                Type(
                    1,
                ),
            ),
            type_length: None,
            repetition_type: Some(
                FieldRepetitionType(
                    0,
                ),
            ),
            name: "a",
            num_children: None,
            converted_type: Some(
                ConvertedType(
                    15,
                ),
            ),
            scale: None,
            precision: None,
            field_id: None,
            logical_type: Some(
                INTEGER(
                    IntType {
                        bit_width: 8,
                        is_signed: true,
                    },
                ),
            ),
        },
        SchemaElement {
            type_: Some(
                Type(
                    6,
                ),
            ),
            type_length: None,
            repetition_type: Some(
                FieldRepetitionType(
                    0,
                ),
            ),
            name: "b",
            num_children: None,
            converted_type: None,
            scale: None,
            precision: None,
            field_id: None,
            logical_type: None,
        },
    ],
    num_rows: 2,
    row_groups: [
        RowGroup {
            columns: [
                ColumnChunk {
                    file_path: None,
                    file_offset: 35,
                    meta_data: Some(
                        ColumnMetaData {
                            type_: Type(
                                1,
                            ),
                            encodings: [
                                Encoding(
                                    0,
                                ),
                                Encoding(
                                    3,
                                ),
                            ],
                            path_in_schema: [
                                "id",
                            ],
                            codec: CompressionCodec(
                                7,
                            ),
                            num_values: 2,
                            total_uncompressed_size: 30,
                            total_compressed_size: 31,
                            key_value_metadata: None,
                            data_page_offset: 4,
                            index_page_offset: None,
                            dictionary_page_offset: None,
                            statistics: None,
                            encoding_stats: None,
                            bloom_filter_offset: None,
                        },
                    ),
                    offset_index_offset: Some(
                        172,
                    ),
                    offset_index_length: Some(
                        10,
                    ),
                    column_index_offset: None,
                    column_index_length: None,
                    crypto_metadata: None,
                    encrypted_column_metadata: None,
                },
                ColumnChunk {
                    file_path: None,
                    file_offset: 88,
                    meta_data: Some(
                        ColumnMetaData {
                            type_: Type(
                                1,
                            ),
                            encodings: [
                                Encoding(
                                    0,
                                ),
                                Encoding(
                                    3,
                                ),
                            ],
                            path_in_schema: [
                                "v",
                                "a",
                            ],
                            codec: CompressionCodec(
                                7,
                            ),
                            num_values: 2,
                            total_uncompressed_size: 30,
                            total_compressed_size: 31,
                            key_value_metadata: None,
                            data_page_offset: 57,
                            index_page_offset: None,
                            dictionary_page_offset: None,
                            statistics: None,
                            encoding_stats: None,
                            bloom_filter_offset: None,
                        },
                    ),
                    offset_index_offset: Some(
                        182,
                    ),
                    offset_index_length: Some(
                        10,
                    ),
                    column_index_offset: None,
                    column_index_length: None,
                    crypto_metadata: None,
                    encrypted_column_metadata: None,
                },
                ColumnChunk {
                    file_path: None,
                    file_offset: 148,
                    meta_data: Some(
                        ColumnMetaData {
                            type_: Type(
                                6,
                            ),
                            encodings: [
                                Encoding(
                                    0,
                                ),
                                Encoding(
                                    3,
                                ),
                            ],
                            path_in_schema: [
                                "v",
                                "b",
                            ],
                            codec: CompressionCodec(
                                7,
                            ),
                            num_values: 2,
                            total_uncompressed_size: 36,
                            total_compressed_size: 37,
                            key_value_metadata: None,
                            data_page_offset: 111,
                            index_page_offset: None,
                            dictionary_page_offset: None,
                            statistics: None,
                            encoding_stats: None,
                            bloom_filter_offset: None,
                        },
                    ),
                    offset_index_offset: Some(
                        192,
                    ),
                    offset_index_length: Some(
                        11,
                    ),
                    column_index_offset: None,
                    column_index_length: None,
                    crypto_metadata: None,
                    encrypted_column_metadata: None,
                },
            ],
            total_byte_size: 96,
            num_rows: 2,
            sorting_columns: None,
            file_offset: Some(
                4,
            ),
            total_compressed_size: Some(
                99,
            ),
            ordinal: Some(
                0,
            ),
        },
    ],
    key_value_metadata: None,
    created_by: Some(
        "Arrow2 - Native Rust implementation of Arrow",
    ),
    column_orders: None,
    encryption_algorithm: None,
    footer_signing_key_metadata: None,
}





























parquet_schema_descriptor=SchemaDescriptor {
    name: "root",
    fields: [
        PrimitiveType(
            PrimitiveType {
                field_info: FieldInfo {
                    name: "id",
                    repetition: Required,
                    id: None,
                },
                logical_type: None,
                converted_type: None,
                physical_type: Int32,
            },
        ),
        GroupType {
            field_info: FieldInfo {
                name: "v",
                repetition: Required,
                id: None,
            },
            logical_type: None,
            converted_type: None,
            fields: [
                PrimitiveType(
                    PrimitiveType {
                        field_info: FieldInfo {
                            name: "a",
                            repetition: Required,
                            id: None,
                        },
                        logical_type: Some(
                            Integer(
                                Int8,
                            ),
                        ),
                        converted_type: Some(
                            Int8,
                        ),
                        physical_type: Int32,
                    },
                ),
                PrimitiveType(
                    PrimitiveType {
                        field_info: FieldInfo {
                            name: "b",
                            repetition: Required,
                            id: None,
                        },
                        logical_type: None,
                        converted_type: None,
                        physical_type: ByteArray,
                    },
                ),
            ],
        },
    ],
    leaves: [
        ColumnDescriptor {
            descriptor: Descriptor {
                primitive_type: PrimitiveType {
                    field_info: FieldInfo {
                        name: "id",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: None,
                    converted_type: None,
                    physical_type: Int32,
                },
                max_def_level: 0,
                max_rep_level: 0,
            },
            path_in_schema: [
                "id",
            ],
            base_type: PrimitiveType(
                PrimitiveType {
                    field_info: FieldInfo {
                        name: "id",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: None,
                    converted_type: None,
                    physical_type: Int32,
                },
            ),
        },
        ColumnDescriptor {
            descriptor: Descriptor {
                primitive_type: PrimitiveType {
                    field_info: FieldInfo {
                        name: "a",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: Some(
                        Integer(
                            Int8,
                        ),
                    ),
                    converted_type: Some(
                        Int8,
                    ),
                    physical_type: Int32,
                },
                max_def_level: 0,
                max_rep_level: 0,
            },
            path_in_schema: [
                "v",
                "a",
            ],
            base_type: GroupType {
                field_info: FieldInfo {
                    name: "v",
                    repetition: Required,
                    id: None,
                },
                logical_type: None,
                converted_type: None,
                fields: [
                    PrimitiveType(
                        PrimitiveType {
                            field_info: FieldInfo {
                                name: "a",
                                repetition: Required,
                                id: None,
                            },
                            logical_type: Some(
                                Integer(
                                    Int8,
                                ),
                            ),
                            converted_type: Some(
                                Int8,
                            ),
                            physical_type: Int32,
                        },
                    ),
                    PrimitiveType(
                        PrimitiveType {
                            field_info: FieldInfo {
                                name: "b",
                                repetition: Required,
                                id: None,
                            },
                            logical_type: None,
                            converted_type: None,
                            physical_type: ByteArray,
                        },
                    ),
                ],
            },
        },
        ColumnDescriptor {
            descriptor: Descriptor {
                primitive_type: PrimitiveType {
                    field_info: FieldInfo {
                        name: "b",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: None,
                    converted_type: None,
                    physical_type: ByteArray,
                },
                max_def_level: 0,
                max_rep_level: 0,
            },
            path_in_schema: [
                "v",
                "b",
            ],
            base_type: GroupType {
                field_info: FieldInfo {
                    name: "v",
                    repetition: Required,
                    id: None,
                },
                logical_type: None,
                converted_type: None,
                fields: [
                    PrimitiveType(
                        PrimitiveType {
                            field_info: FieldInfo {
                                name: "a",
                                repetition: Required,
                                id: None,
                            },
                            logical_type: Some(
                                Integer(
                                    Int8,
                                ),
                            ),
                            converted_type: Some(
                                Int8,
                            ),
                            physical_type: Int32,
                        },
                    ),
                    PrimitiveType(
                        PrimitiveType {
                            field_info: FieldInfo {
                                name: "b",
                                repetition: Required,
                                id: None,
                            },
                            logical_type: None,
                            converted_type: None,
                            physical_type: ByteArray,
                        },
                    ),
                ],
            },
        },
    ],
}


projection=[
    0,
    1,
]












parquet_schema_descriptor=SchemaDescriptor {
    name: "root",
    fields: [
        PrimitiveType(
            PrimitiveType {
                field_info: FieldInfo {
                    name: "a",
                    repetition: Required,
                    id: None,
                },
                logical_type: Some(
                    Integer(
                        Int8,
                    ),
                ),
                converted_type: Some(
                    Int8,
                ),
                physical_type: Int32,
            },
        ),
        PrimitiveType(
            PrimitiveType {
                field_info: FieldInfo {
                    name: "b",
                    repetition: Required,
                    id: None,
                },
                logical_type: None,
                converted_type: None,
                physical_type: ByteArray,
            },
        ),
    ],
    leaves: [
        ColumnDescriptor {
            descriptor: Descriptor {
                primitive_type: PrimitiveType {
                    field_info: FieldInfo {
                        name: "a",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: Some(
                        Integer(
                            Int8,
                        ),
                    ),
                    converted_type: Some(
                        Int8,
                    ),
                    physical_type: Int32,
                },
                max_def_level: 0,
                max_rep_level: 0,
            },
            path_in_schema: [
                "a",
            ],
            base_type: PrimitiveType(
                PrimitiveType {
                    field_info: FieldInfo {
                        name: "a",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: Some(
                        Integer(
                            Int8,
                        ),
                    ),
                    converted_type: Some(
                        Int8,
                    ),
                    physical_type: Int32,
                },
            ),
        },
        ColumnDescriptor {
            descriptor: Descriptor {
                primitive_type: PrimitiveType {
                    field_info: FieldInfo {
                        name: "b",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: None,
                    converted_type: None,
                    physical_type: ByteArray,
                },
                max_def_level: 0,
                max_rep_level: 0,
            },
            path_in_schema: [
                "b",
            ],
            base_type: PrimitiveType(
                PrimitiveType {
                    field_info: FieldInfo {
                        name: "b",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: None,
                    converted_type: None,
                    physical_type: ByteArray,
                },
            ),
        },
    ],
}












parquet_schema_descriptor=SchemaDescriptor {
    name: "root",
    fields: [
        PrimitiveType(
            PrimitiveType {
                field_info: FieldInfo {
                    name: "id",
                    repetition: Required,
                    id: None,
                },
                logical_type: None,
                converted_type: None,
                physical_type: Int32,
            },
        ),
        GroupType {
            field_info: FieldInfo {
                name: "v",
                repetition: Required,
                id: None,
            },
            logical_type: None,
            converted_type: None,
            fields: [
                PrimitiveType(
                    PrimitiveType {
                        field_info: FieldInfo {
                            name: "a",
                            repetition: Required,
                            id: None,
                        },
                        logical_type: Some(
                            Integer(
                                Int8,
                            ),
                        ),
                        converted_type: Some(
                            Int8,
                        ),
                        physical_type: Int32,
                    },
                ),
                PrimitiveType(
                    PrimitiveType {
                        field_info: FieldInfo {
                            name: "b",
                            repetition: Required,
                            id: None,
                        },
                        logical_type: None,
                        converted_type: None,
                        physical_type: ByteArray,
                    },
                ),
            ],
        },
        GroupType {
            field_info: FieldInfo {
                name: "z",
                repetition: Required,
                id: None,
            },
            logical_type: None,
            converted_type: None,
            fields: [
                PrimitiveType(
                    PrimitiveType {
                        field_info: FieldInfo {
                            name: "c",
                            repetition: Required,
                            id: None,
                        },
                        logical_type: None,
                        converted_type: None,
                        physical_type: Int32,
                    },
                ),
                PrimitiveType(
                    PrimitiveType {
                        field_info: FieldInfo {
                            name: "d",
                            repetition: Required,
                            id: None,
                        },
                        logical_type: None,
                        converted_type: None,
                        physical_type: ByteArray,
                    },
                ),
            ],
        },
    ],
    leaves: [
        ColumnDescriptor {
            descriptor: Descriptor {
                primitive_type: PrimitiveType {
                    field_info: FieldInfo {
                        name: "id",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: None,
                    converted_type: None,
                    physical_type: Int32,
                },
                max_def_level: 0,
                max_rep_level: 0,
            },
            path_in_schema: [
                "id",
            ],
            base_type: PrimitiveType(
                PrimitiveType {
                    field_info: FieldInfo {
                        name: "id",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: None,
                    converted_type: None,
                    physical_type: Int32,
                },
            ),
        },
        ColumnDescriptor {
            descriptor: Descriptor {
                primitive_type: PrimitiveType {
                    field_info: FieldInfo {
                        name: "a",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: Some(
                        Integer(
                            Int8,
                        ),
                    ),
                    converted_type: Some(
                        Int8,
                    ),
                    physical_type: Int32,
                },
                max_def_level: 0,
                max_rep_level: 0,
            },
            path_in_schema: [
                "v",
                "a",
            ],
            base_type: GroupType {
                field_info: FieldInfo {
                    name: "v",
                    repetition: Required,
                    id: None,
                },
                logical_type: None,
                converted_type: None,
                fields: [
                    PrimitiveType(
                        PrimitiveType {
                            field_info: FieldInfo {
                                name: "a",
                                repetition: Required,
                                id: None,
                            },
                            logical_type: Some(
                                Integer(
                                    Int8,
                                ),
                            ),
                            converted_type: Some(
                                Int8,
                            ),
                            physical_type: Int32,
                        },
                    ),
                    PrimitiveType(
                        PrimitiveType {
                            field_info: FieldInfo {
                                name: "b",
                                repetition: Required,
                                id: None,
                            },
                            logical_type: None,
                            converted_type: None,
                            physical_type: ByteArray,
                        },
                    ),
                ],
            },
        },
        ColumnDescriptor {
            descriptor: Descriptor {
                primitive_type: PrimitiveType {
                    field_info: FieldInfo {
                        name: "b",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: None,
                    converted_type: None,
                    physical_type: ByteArray,
                },
                max_def_level: 0,
                max_rep_level: 0,
            },
            path_in_schema: [
                "v",
                "b",
            ],
            base_type: GroupType {
                field_info: FieldInfo {
                    name: "v",
                    repetition: Required,
                    id: None,
                },
                logical_type: None,
                converted_type: None,
                fields: [
                    PrimitiveType(
                        PrimitiveType {
                            field_info: FieldInfo {
                                name: "a",
                                repetition: Required,
                                id: None,
                            },
                            logical_type: Some(
                                Integer(
                                    Int8,
                                ),
                            ),
                            converted_type: Some(
                                Int8,
                            ),
                            physical_type: Int32,
                        },
                    ),
                    PrimitiveType(
                        PrimitiveType {
                            field_info: FieldInfo {
                                name: "b",
                                repetition: Required,
                                id: None,
                            },
                            logical_type: None,
                            converted_type: None,
                            physical_type: ByteArray,
                        },
                    ),
                ],
            },
        },
        ColumnDescriptor {
            descriptor: Descriptor {
                primitive_type: PrimitiveType {
                    field_info: FieldInfo {
                        name: "c",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: None,
                    converted_type: None,
                    physical_type: Int32,
                },
                max_def_level: 0,
                max_rep_level: 0,
            },
            path_in_schema: [
                "z",
                "c",
            ],
            base_type: GroupType {
                field_info: FieldInfo {
                    name: "z",
                    repetition: Required,
                    id: None,
                },
                logical_type: None,
                converted_type: None,
                fields: [
                    PrimitiveType(
                        PrimitiveType {
                            field_info: FieldInfo {
                                name: "c",
                                repetition: Required,
                                id: None,
                            },
                            logical_type: None,
                            converted_type: None,
                            physical_type: Int32,
                        },
                    ),
                    PrimitiveType(
                        PrimitiveType {
                            field_info: FieldInfo {
                                name: "d",
                                repetition: Required,
                                id: None,
                            },
                            logical_type: None,
                            converted_type: None,
                            physical_type: ByteArray,
                        },
                    ),
                ],
            },
        },
        ColumnDescriptor {
            descriptor: Descriptor {
                primitive_type: PrimitiveType {
                    field_info: FieldInfo {
                        name: "d",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: None,
                    converted_type: None,
                    physical_type: ByteArray,
                },
                max_def_level: 0,
                max_rep_level: 0,
            },
            path_in_schema: [
                "z",
                "d",
            ],
            base_type: GroupType {
                field_info: FieldInfo {
                    name: "z",
                    repetition: Required,
                    id: None,
                },
                logical_type: None,
                converted_type: None,
                fields: [
                    PrimitiveType(
                        PrimitiveType {
                            field_info: FieldInfo {
                                name: "c",
                                repetition: Required,
                                id: None,
                            },
                            logical_type: None,
                            converted_type: None,
                            physical_type: Int32,
                        },
                    ),
                    PrimitiveType(
                        PrimitiveType {
                            field_info: FieldInfo {
                                name: "d",
                                repetition: Required,
                                id: None,
                            },
                            logical_type: None,
                            converted_type: None,
                            physical_type: ByteArray,
                        },
                    ),
                ],
            },
        },
    ],
}





















projection_partitions---------
indices=[0, 1]
part columns_meta={0: ColumnMeta { offset: 4, length: 31, num_values: 2 }, 1: ColumnMeta { offset: 57, length: 31, num_values: 2 }}
part columns_path=Some({"id": 0, "s:b": 3, "s:a:c": 1, "s:a:d": 2})
arrow_schema=Schema {
    fields: [
        Field {
            name: "id",
            data_type: Int32,
            is_nullable: false,
            metadata: {},
        },
        Field {
            name: "s",
            data_type: Struct(
                [
                    Field {
                        name: "a",
                        data_type: Struct(
                            [
                                Field {
                                    name: "c",
                                    data_type: Int32,
                                    is_nullable: false,
                                    metadata: {},
                                },
                                Field {
                                    name: "d",
                                    data_type: Int32,
                                    is_nullable: false,
                                    metadata: {},
                                },
                            ],
                        ),
                        is_nullable: false,
                        metadata: {},
                    },
                    Field {
                        name: "b",
                        data_type: Int32,
                        is_nullable: false,
                        metadata: {},
                    },
                ],
            ),
            is_nullable: false,
            metadata: {},
        },
    ],
    metadata: {},
}
parquet_schema_descriptor=SchemaDescriptor {
    name: "root",
    fields: [
        PrimitiveType(
            PrimitiveType {
                field_info: FieldInfo {
                    name: "id",
                    repetition: Required,
                    id: None,
                },
                logical_type: None,
                converted_type: None,
                physical_type: Int32,
            },
        ),
        GroupType {
            field_info: FieldInfo {
                name: "s",
                repetition: Required,
                id: None,
            },
            logical_type: None,
            converted_type: None,
            fields: [
                GroupType {
                    field_info: FieldInfo {
                        name: "a",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: None,
                    converted_type: None,
                    fields: [
                        PrimitiveType(
                            PrimitiveType {
                                field_info: FieldInfo {
                                    name: "c",
                                    repetition: Required,
                                    id: None,
                                },
                                logical_type: None,
                                converted_type: None,
                                physical_type: Int32,
                            },
                        ),
                        PrimitiveType(
                            PrimitiveType {
                                field_info: FieldInfo {
                                    name: "d",
                                    repetition: Required,
                                    id: None,
                                },
                                logical_type: None,
                                converted_type: None,
                                physical_type: Int32,
                            },
                        ),
                    ],
                },
                PrimitiveType(
                    PrimitiveType {
                        field_info: FieldInfo {
                            name: "b",
                            repetition: Required,
                            id: None,
                        },
                        logical_type: None,
                        converted_type: None,
                        physical_type: Int32,
                    },
                ),
            ],
        },
    ],
    leaves: [
        ColumnDescriptor {
            descriptor: Descriptor {
                primitive_type: PrimitiveType {
                    field_info: FieldInfo {
                        name: "id",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: None,
                    converted_type: None,
                    physical_type: Int32,
                },
                max_def_level: 0,
                max_rep_level: 0,
            },
            path_in_schema: [
                "id",
            ],
            base_type: PrimitiveType(
                PrimitiveType {
                    field_info: FieldInfo {
                        name: "id",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: None,
                    converted_type: None,
                    physical_type: Int32,
                },
            ),
        },
        ColumnDescriptor {
            descriptor: Descriptor {
                primitive_type: PrimitiveType {
                    field_info: FieldInfo {
                        name: "c",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: None,
                    converted_type: None,
                    physical_type: Int32,
                },
                max_def_level: 0,
                max_rep_level: 0,
            },
            path_in_schema: [
                "s",
                "a",
                "c",
            ],
            base_type: GroupType {
                field_info: FieldInfo {
                    name: "s",
                    repetition: Required,
                    id: None,
                },
                logical_type: None,
                converted_type: None,
                fields: [
                    GroupType {
                        field_info: FieldInfo {
                            name: "a",
                            repetition: Required,
                            id: None,
                        },
                        logical_type: None,
                        converted_type: None,
                        fields: [
                            PrimitiveType(
                                PrimitiveType {
                                    field_info: FieldInfo {
                                        name: "c",
                                        repetition: Required,
                                        id: None,
                                    },
                                    logical_type: None,
                                    converted_type: None,
                                    physical_type: Int32,
                                },
                            ),
                            PrimitiveType(
                                PrimitiveType {
                                    field_info: FieldInfo {
                                        name: "d",
                                        repetition: Required,
                                        id: None,
                                    },
                                    logical_type: None,
                                    converted_type: None,
                                    physical_type: Int32,
                                },
                            ),
                        ],
                    },
                    PrimitiveType(
                        PrimitiveType {
                            field_info: FieldInfo {
                                name: "b",
                                repetition: Required,
                                id: None,
                            },
                            logical_type: None,
                            converted_type: None,
                            physical_type: Int32,
                        },
                    ),
                ],
            },
        },
        ColumnDescriptor {
            descriptor: Descriptor {
                primitive_type: PrimitiveType {
                    field_info: FieldInfo {
                        name: "d",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: None,
                    converted_type: None,
                    physical_type: Int32,
                },
                max_def_level: 0,
                max_rep_level: 0,
            },
            path_in_schema: [
                "s",
                "a",
                "d",
            ],
            base_type: GroupType {
                field_info: FieldInfo {
                    name: "s",
                    repetition: Required,
                    id: None,
                },
                logical_type: None,
                converted_type: None,
                fields: [
                    GroupType {
                        field_info: FieldInfo {
                            name: "a",
                            repetition: Required,
                            id: None,
                        },
                        logical_type: None,
                        converted_type: None,
                        fields: [
                            PrimitiveType(
                                PrimitiveType {
                                    field_info: FieldInfo {
                                        name: "c",
                                        repetition: Required,
                                        id: None,
                                    },
                                    logical_type: None,
                                    converted_type: None,
                                    physical_type: Int32,
                                },
                            ),
                            PrimitiveType(
                                PrimitiveType {
                                    field_info: FieldInfo {
                                        name: "d",
                                        repetition: Required,
                                        id: None,
                                    },
                                    logical_type: None,
                                    converted_type: None,
                                    physical_type: Int32,
                                },
                            ),
                        ],
                    },
                    PrimitiveType(
                        PrimitiveType {
                            field_info: FieldInfo {
                                name: "b",
                                repetition: Required,
                                id: None,
                            },
                            logical_type: None,
                            converted_type: None,
                            physical_type: Int32,
                        },
                    ),
                ],
            },
        },
        ColumnDescriptor {
            descriptor: Descriptor {
                primitive_type: PrimitiveType {
                    field_info: FieldInfo {
                        name: "b",
                        repetition: Required,
                        id: None,
                    },
                    logical_type: None,
                    converted_type: None,
                    physical_type: Int32,
                },
                max_def_level: 0,
                max_rep_level: 0,
            },
            path_in_schema: [
                "s",
                "b",
            ],
            base_type: GroupType {
                field_info: FieldInfo {
                    name: "s",
                    repetition: Required,
                    id: None,
                },
                logical_type: None,
                converted_type: None,
                fields: [
                    GroupType {
                        field_info: FieldInfo {
                            name: "a",
                            repetition: Required,
                            id: None,
                        },
                        logical_type: None,
                        converted_type: None,
                        fields: [
                            PrimitiveType(
                                PrimitiveType {
                                    field_info: FieldInfo {
                                        name: "c",
                                        repetition: Required,
                                        id: None,
                                    },
                                    logical_type: None,
                                    converted_type: None,
                                    physical_type: Int32,
                                },
                            ),
                            PrimitiveType(
                                PrimitiveType {
                                    field_info: FieldInfo {
                                        name: "d",
                                        repetition: Required,
                                        id: None,
                                    },
                                    logical_type: None,
                                    converted_type: None,
                                    physical_type: Int32,
                                },
                            ),
                        ],
                    },
                    PrimitiveType(
                        PrimitiveType {
                            field_info: FieldInfo {
                                name: "b",
                                repetition: Required,
                                id: None,
                            },
                            logical_type: None,
                            converted_type: None,
                            physical_type: Int32,
                        },
                    ),
                ],
            },
        },
    ],
}
projection=[
    0,
    1,
]
columns.len()=1
types.len()=1
n==1
n==2


















file_meta=FileMetaData {
    version: 2,
    schema: [
        SchemaElement {
            type_: None,
            type_length: None,
            repetition_type: None,
            name: "root",
            num_children: Some(
                2,
            ),
            converted_type: None,
            scale: None,
            precision: None,
            field_id: None,
            logical_type: None,
        },
        SchemaElement {
            type_: Some(
                Type(
                    1,
                ),
            ),
            type_length: None,
            repetition_type: Some(
                FieldRepetitionType(
                    0,
                ),
            ),
            name: "id",
            num_children: None,
            converted_type: None,
            scale: None,
            precision: None,
            field_id: None,
            logical_type: None,
        },
        SchemaElement {
            type_: None,
            type_length: None,
            repetition_type: Some(
                FieldRepetitionType(
                    0,
                ),
            ),
            name: "s",
            num_children: Some(
                2,
            ),
            converted_type: None,
            scale: None,
            precision: None,
            field_id: None,
            logical_type: None,
        },
        SchemaElement {
            type_: None,
            type_length: None,
            repetition_type: Some(
                FieldRepetitionType(
                    0,
                ),
            ),
            name: "a",
            num_children: Some(
                2,
            ),
            converted_type: None,
            scale: None,
            precision: None,
            field_id: None,
            logical_type: None,
        },
        SchemaElement {
            type_: Some(
                Type(
                    1,
                ),
            ),
            type_length: None,
            repetition_type: Some(
                FieldRepetitionType(
                    0,
                ),
            ),
            name: "c",
            num_children: None,
            converted_type: None,
            scale: None,
            precision: None,
            field_id: None,
            logical_type: None,
        },
        SchemaElement {
            type_: Some(
                Type(
                    1,
                ),
            ),
            type_length: None,
            repetition_type: Some(
                FieldRepetitionType(
                    0,
                ),
            ),
            name: "d",
            num_children: None,
            converted_type: None,
            scale: None,
            precision: None,
            field_id: None,
            logical_type: None,
        },
        SchemaElement {
            type_: Some(
                Type(
                    1,
                ),
            ),
            type_length: None,
            repetition_type: Some(
                FieldRepetitionType(
                    0,
                ),
            ),
            name: "b",
            num_children: None,
            converted_type: None,
            scale: None,
            precision: None,
            field_id: None,
            logical_type: None,
        },
    ],
    num_rows: 2,
    row_groups: [
        RowGroup {
            columns: [
                ColumnChunk {
                    file_path: None,
                    file_offset: 35,
                    meta_data: Some(
                        ColumnMetaData {
                            type_: Type(
                                1,
                            ),
                            encodings: [
                                Encoding(
                                    0,
                                ),
                                Encoding(
                                    3,
                                ),
                            ],
                            path_in_schema: [
                                "id",
                            ],
                            codec: CompressionCodec(
                                7,
                            ),
                            num_values: 2,
                            total_uncompressed_size: 30,
                            total_compressed_size: 31,
                            key_value_metadata: None,
                            data_page_offset: 4,
                            index_page_offset: None,
                            dictionary_page_offset: None,
                            statistics: None,
                            encoding_stats: None,
                            bloom_filter_offset: None,
                        },
                    ),
                    offset_index_offset: Some(
                        225,
                    ),
                    offset_index_length: Some(
                        10,
                    ),
                    column_index_offset: None,
                    column_index_length: None,
                    crypto_metadata: None,
                    encrypted_column_metadata: None,
                },
                ColumnChunk {
                    file_path: None,
                    file_offset: 88,
                    meta_data: Some(
                        ColumnMetaData {
                            type_: Type(
                                1,
                            ),
                            encodings: [
                                Encoding(
                                    0,
                                ),
                                Encoding(
                                    3,
                                ),
                            ],
                            path_in_schema: [
                                "s",
                                "a",
                                "c",
                            ],
                            codec: CompressionCodec(
                                7,
                            ),
                            num_values: 2,
                            total_uncompressed_size: 30,
                            total_compressed_size: 31,
                            key_value_metadata: None,
                            data_page_offset: 57,
                            index_page_offset: None,
                            dictionary_page_offset: None,
                            statistics: None,
                            encoding_stats: None,
                            bloom_filter_offset: None,
                        },
                    ),
                    offset_index_offset: Some(
                        235,
                    ),
                    offset_index_length: Some(
                        10,
                    ),
                    column_index_offset: None,
                    column_index_length: None,
                    crypto_metadata: None,
                    encrypted_column_metadata: None,
                },
                ColumnChunk {
                    file_path: None,
                    file_offset: 144,
                    meta_data: Some(
                        ColumnMetaData {
                            type_: Type(
                                1,
                            ),
                            encodings: [
                                Encoding(
                                    0,
                                ),
                                Encoding(
                                    3,
                                ),
                            ],
                            path_in_schema: [
                                "s",
                                "a",
                                "d",
                            ],
                            codec: CompressionCodec(
                                7,
                            ),
                            num_values: 2,
                            total_uncompressed_size: 30,
                            total_compressed_size: 31,
                            key_value_metadata: None,
                            data_page_offset: 113,
                            index_page_offset: None,
                            dictionary_page_offset: None,
                            statistics: None,
                            encoding_stats: None,
                            bloom_filter_offset: None,
                        },
                    ),
                    offset_index_offset: Some(
                        245,
                    ),
                    offset_index_length: Some(
                        11,
                    ),
                    column_index_offset: None,
                    column_index_length: None,
                    crypto_metadata: None,
                    encrypted_column_metadata: None,
                },
                ColumnChunk {
                    file_path: None,
                    file_offset: 201,
                    meta_data: Some(
                        ColumnMetaData {
                            type_: Type(
                                1,
                            ),
                            encodings: [
                                Encoding(
                                    0,
                                ),
                                Encoding(
                                    3,
                                ),
                            ],
                            path_in_schema: [
                                "s",
                                "b",
                            ],
                            codec: CompressionCodec(
                                7,
                            ),
                            num_values: 2,
                            total_uncompressed_size: 30,
                            total_compressed_size: 31,
                            key_value_metadata: None,
                            data_page_offset: 170,
                            index_page_offset: None,
                            dictionary_page_offset: None,
                            statistics: None,
                            encoding_stats: None,
                            bloom_filter_offset: None,
                        },
                    ),
                    offset_index_offset: Some(
                        256,
                    ),
                    offset_index_length: Some(
                        11,
                    ),
                    column_index_offset: None,
                    column_index_length: None,
                    crypto_metadata: None,
                    encrypted_column_metadata: None,
                },
            ],
            total_byte_size: 120,
            num_rows: 2,
            sorting_columns: None,
            file_offset: Some(
                4,
            ),
            total_compressed_size: Some(
                124,
            ),
            ordinal: Some(
                0,
            ),
        },
    ],
    key_value_metadata: None,
    created_by: Some(
        "Arrow2 - Native Rust implementation of Arrow",
    ),
    column_orders: None,
    encryption_algorithm: None,
    footer_signing_key_metadata: None,
}





      1
    /   \
   2     7
  / \   / \
 3   6 8   9
/ \
4  5








node_stack=[ColumnSchema { name: "root", column_id: None, children: None }]

node_stack=[ColumnSchema { name: "root", column_id: None, children: Some([ColumnSchema { name: "id", column_id: Some(0), children: None }]) }]

node_stack=[ColumnSchema { name: "root", column_id: None, children: Some([ColumnSchema { name: "id", column_id: Some(0), children: None }, ColumnSchema { name: "s", column_id: None, children: None }]) }, ColumnSchema { name: "s", column_id: None, children: None }]

node_stack=[ColumnSchema { name: "root", column_id: None, children: Some([ColumnSchema { name: "id", column_id: Some(0), children: None }, ColumnSchema { name: "s", column_id: None, children: None }]) }, ColumnSchema { name: "s", column_id: None, children: Some([ColumnSchema { name: "a", column_id: None, children: None }]) }, ColumnSchema { name: "a", column_id: None, children: None }]

node_stack=[ColumnSchema { name: "root", column_id: None, children: Some([ColumnSchema { name: "id", column_id: Some(0), children: None }, ColumnSchema { name: "s", column_id: None, children: None }]) }, ColumnSchema { name: "s", column_id: None, children: Some([ColumnSchema { name: "a", column_id: None, children: None }]) }, ColumnSchema { name: "a", column_id: None, children: Some([ColumnSchema { name: "c", column_id: Some(1), children: None }]) }]

node_stack=[ColumnSchema { name: "root", column_id: None, children: Some([ColumnSchema { name: "id", column_id: Some(0), children: None }, ColumnSchema { name: "s", column_id: None, children: None }]) }, ColumnSchema { name: "s", column_id: None, children: Some([ColumnSchema { name: "a", column_id: None, children: None }]) }, ColumnSchema { name: "a", column_id: None, children: Some([ColumnSchema { name: "c", column_id: Some(1), children: None }, ColumnSchema { name: "d", column_id: Some(2), children: None }]) }]

node_stack=[ColumnSchema { name: "root", column_id: None, children: Some([ColumnSchema { name: "id", column_id: Some(0), children: None }, ColumnSchema { name: "s", column_id: None, children: None }]) }, ColumnSchema { name: "s", column_id: None, children: Some([ColumnSchema { name: "a", column_id: None, children: None }]) }]





part columns_meta={
    1: ColumnMeta { offset: 170, length: 31, num_values: 2 },
    0: ColumnMeta { offset: 4, length: 31, num_values: 2 }
}






