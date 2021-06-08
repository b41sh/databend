// Copyright 2020-2021 The Datafuse Authors.
//
// SPDX-License-Identifier: Apache-2.0.

use std::convert::From;
use std::convert::TryFrom;

use common_exception::ErrorCodes;
use common_exception::Result;
use ordered_float::OrderedFloat;

use crate::DataValue;

/// Enumeration of types that can be used in a GROUP BY expression
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum DataGroupValue {
    Float32(OrderedFloat<f32>),
    Float64(OrderedFloat<f64>),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Utf8(Box<String>),
    Boolean(bool),
    TimeMillisecond(i64),
    TimeMicrosecond(i64),
    TimeNanosecond(i64),
    Date32(i32),
    Date64(i64),
}

impl TryFrom<&DataValue> for DataGroupValue {
    type Error = ErrorCodes;

    fn try_from(value: &DataValue) -> Result<Self> {
        Ok(match value {
            DataValue::Float32(Some(v)) => DataGroupValue::Float32(OrderedFloat::from(*v)),
            DataValue::Float64(Some(v)) => DataGroupValue::Float64(OrderedFloat::from(*v)),
            DataValue::Boolean(Some(v)) => DataGroupValue::Boolean(*v),
            DataValue::Int8(Some(v)) => DataGroupValue::Int8(*v),
            DataValue::Int16(Some(v)) => DataGroupValue::Int16(*v),
            DataValue::Int32(Some(v)) => DataGroupValue::Int32(*v),
            DataValue::Int64(Some(v)) => DataGroupValue::Int64(*v),
            DataValue::UInt8(Some(v)) => DataGroupValue::UInt8(*v),
            DataValue::UInt16(Some(v)) => DataGroupValue::UInt16(*v),
            DataValue::UInt32(Some(v)) => DataGroupValue::UInt32(*v),
            DataValue::UInt64(Some(v)) => DataGroupValue::UInt64(*v),
            DataValue::TimestampMillisecond(Some(v)) => DataGroupValue::TimeMillisecond(*v),
            DataValue::TimestampMicrosecond(Some(v)) => DataGroupValue::TimeMicrosecond(*v),
            DataValue::TimestampNanosecond(Some(v)) => DataGroupValue::TimeNanosecond(*v),
            DataValue::Utf8(Some(v)) => DataGroupValue::Utf8(Box::new(v.clone())),
            DataValue::Date32(Some(v)) => DataGroupValue::Date32(*v),
            DataValue::Date64(Some(v)) => DataGroupValue::Date64(*v),

            DataValue::Float32(None)
            | DataValue::Float64(None)
            | DataValue::Boolean(None)
            | DataValue::Int8(None)
            | DataValue::Int16(None)
            | DataValue::Int32(None)
            | DataValue::Int64(None)
            | DataValue::UInt8(None)
            | DataValue::UInt16(None)
            | DataValue::UInt32(None)
            | DataValue::UInt64(None)
            | DataValue::Utf8(None) => {
                return Err(ErrorCodes::BadDataValueType(format!(
                    "Cannot convert a DataValue holding NULL ({:?})",
                    value
                )));
            }

            v => {
                return Err(ErrorCodes::BadDataValueType(format!(
                    "Cannot convert a DataValue  with associated DataType ({:?})",
                    v.data_type()
                )));
            }
        })
    }
}

impl From<&DataGroupValue> for DataValue {
    fn from(group_by_scalar: &DataGroupValue) -> Self {
        match group_by_scalar {
            DataGroupValue::Float32(v) => DataValue::Float32(Some((*v).into())),
            DataGroupValue::Float64(v) => DataValue::Float64(Some((*v).into())),
            DataGroupValue::Boolean(v) => DataValue::Boolean(Some(*v)),
            DataGroupValue::Int8(v) => DataValue::Int8(Some(*v)),
            DataGroupValue::Int16(v) => DataValue::Int16(Some(*v)),
            DataGroupValue::Int32(v) => DataValue::Int32(Some(*v)),
            DataGroupValue::Int64(v) => DataValue::Int64(Some(*v)),
            DataGroupValue::UInt8(v) => DataValue::UInt8(Some(*v)),
            DataGroupValue::UInt16(v) => DataValue::UInt16(Some(*v)),
            DataGroupValue::UInt32(v) => DataValue::UInt32(Some(*v)),
            DataGroupValue::UInt64(v) => DataValue::UInt64(Some(*v)),
            DataGroupValue::Utf8(v) => DataValue::Utf8(Some(v.to_string())),
            DataGroupValue::TimeMillisecond(v) => DataValue::TimestampMillisecond(Some(*v)),
            DataGroupValue::TimeMicrosecond(v) => DataValue::TimestampMicrosecond(Some(*v)),
            DataGroupValue::TimeNanosecond(v) => DataValue::TimestampNanosecond(Some(*v)),
            DataGroupValue::Date32(v) => DataValue::Date32(Some(*v)),
            DataGroupValue::Date64(v) => DataValue::Date64(Some(*v)),
        }
    }
}
