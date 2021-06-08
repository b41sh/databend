// Copyright 2020-2021 The Datafuse Authors.
//
// SPDX-License-Identifier: Apache-2.0.

use std::any::Any;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt;

use common_arrow::arrow::array::Array;
use common_datavalues::downcast_array;
use common_datavalues::*;
use common_exception::ErrorCodes;
use common_exception::Result;

use crate::aggregator_common::assert_binary_arguments;
use crate::IAggregateFunction;

#[derive(Clone)]
pub struct AggregateArgMaxFunction {
    display_name: String,
    state: DataValue,
    arguments: Vec<DataField>,
}

impl AggregateArgMaxFunction {
    pub fn try_create(
        display_name: &str,
        arguments: Vec<DataField>,
    ) -> Result<Box<dyn IAggregateFunction>> {
        assert_binary_arguments(display_name, arguments.len())?;

        Ok(Box::new(AggregateArgMaxFunction {
            display_name: display_name.to_string(),
            state: DataValue::Struct(vec![
                DataValue::try_from(arguments[0].data_type())?,
                DataValue::try_from(arguments[1].data_type())?,
            ]),
            arguments,
        }))
    }
}

impl IAggregateFunction for AggregateArgMaxFunction {
    fn name(&self) -> &str {
        "AggregateArgMaxFunction"
    }

    fn return_type(&self) -> Result<DataType> {
        Ok(self.arguments[0].data_type().clone())
    }

    fn nullable(&self, _input_schema: &DataSchema) -> Result<bool> {
        Ok(false)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn accumulate(&mut self, columns: &[DataColumnarValue], _input_rows: usize) -> Result<()> {
        if let DataValue::Struct(max_arg_val) = Self::arg_max_batch(columns[1].clone())? {
            let index: u64 = max_arg_val[0].clone().try_into()?;
            let max_arg = DataValue::try_from_array(&columns[0].to_array()?, index as usize)?;
            let max_val = max_arg_val[1].clone();

            if let DataValue::Struct(old_max_arg_val) = self.state.clone() {
                let old_max_arg = old_max_arg_val[0].clone();
                let old_max_val = old_max_arg_val[1].clone();
                let new_max_val = DataValueAggregate::data_value_aggregate_op(
                    DataValueAggregateOperator::Max,
                    old_max_val.clone(),
                    max_val,
                )?;
                self.state = DataValue::Struct(vec![
                    if new_max_val == old_max_val {
                        old_max_arg
                    } else {
                        max_arg
                    },
                    new_max_val,
                ]);
            }
        }
        Ok(())
    }

    fn accumulate_scalar(&mut self, values: &[DataValue]) -> Result<()> {
        if let DataValue::Struct(old_max_arg_val) = self.state.clone() {
            let old_max_arg = old_max_arg_val[0].clone();
            let old_max_val = old_max_arg_val[1].clone();
            let new_max_val = DataValueAggregate::data_value_aggregate_op(
                DataValueAggregateOperator::Max,
                old_max_val.clone(),
                values[1].clone(),
            )?;
            self.state = DataValue::Struct(vec![
                if new_max_val == old_max_val {
                    old_max_arg
                } else {
                    values[0].clone()
                },
                new_max_val,
            ]);
        }
        Ok(())
    }

    fn accumulate_result(&self) -> Result<Vec<DataValue>> {
        Ok(vec![self.state.clone()])
    }

    fn merge(&mut self, states: &[DataValue]) -> Result<()> {
        let arg_val = states[0].clone();
        if let (DataValue::Struct(new_states), DataValue::Struct(old_states)) =
            (arg_val, self.state.clone())
        {
            let new_max_val = DataValueAggregate::data_value_aggregate_op(
                DataValueAggregateOperator::Max,
                new_states[1].clone(),
                old_states[1].clone(),
            )?;
            self.state = DataValue::Struct(vec![
                if new_max_val == old_states[1] {
                    old_states[0].clone()
                } else {
                    new_states[0].clone()
                },
                new_max_val,
            ]);
        }
        Ok(())
    }

    fn merge_result(&self) -> Result<DataValue> {
        Ok(if let DataValue::Struct(state) = self.state.clone() {
            state[0].clone()
        } else {
            self.state.clone()
        })
    }
}

impl fmt::Display for AggregateArgMaxFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.display_name)
    }
}

macro_rules! typed_array_max_to_data_value {
    ($VALUES:expr, $ARRAYTYPE:ident, $SCALAR:ident, $OP:expr $(,)?) => {{
        let array = downcast_array!($VALUES, $ARRAYTYPE)?;
        let values = array.values();
        let data = array.data();
        let null_count = array.null_count();
        let mut max_row_val = (0, values[0]);

        if null_count == 0 {
            for row in 1..data.len() {
                if values[row] > max_row_val.1 {
                    max_row_val = (row, values[row]);
                }
            }
        } else {
            for row in 1..data.len() {
                if data.is_valid(row) && values[row] > max_row_val.1 {
                    max_row_val = (row, values[row]);
                }
            }
        }

        Result::Ok(DataValue::Struct(vec![
            DataValue::UInt64(Some(max_row_val.0 as u64)),
            DataValue::$SCALAR(Some(max_row_val.1)),
        ]))
    }};
}

macro_rules! string_array_max_to_data_value {
    ($VALUES:expr, $ARRAYTYPE:ident, $SCALAR:ident, $OP:expr $(,)?) => {{
        fn cmp(a: &str, b: &str) -> bool {
            a < b
        }
        let array = downcast_array!($VALUES, $ARRAYTYPE)?;
        let data = array.data();

        let null_count = array.null_count();
        let mut max_row_val = (0usize, array.value(0));

        if null_count == 0 {
            for row in 1..data.len() {
                let item = array.value(row);
                if cmp(&max_row_val.1, item) {
                    max_row_val = (row, item);
                }
            }
        } else {
            for row in 1..data.len() {
                let item = array.value(row);
                if data.is_valid(row) && cmp(&max_row_val.1, item) {
                    max_row_val = (row, item);
                }
            }
        }

        Result::Ok(DataValue::Struct(vec![
            DataValue::UInt64(Some(max_row_val.0 as u64)),
            DataValue::$SCALAR(Some(max_row_val.1.to_string())),
        ]))
    }};
}

impl AggregateArgMaxFunction {
    pub fn arg_max_batch(column: DataColumnarValue) -> Result<DataValue> {
        match column {
            DataColumnarValue::Constant(value, _) => {
                Ok(DataValue::Struct(vec![DataValue::UInt64(Some(0)), value]))
            }

            DataColumnarValue::Array(array) => {
                if let Ok(v) =
                    dispatch_primitive_array! { typed_array_max_to_data_value, array, argMax}
                {
                    Ok(v)
                } else {
                    dispatch_string_array! { string_array_max_to_data_value, array, argMax}
                }
            }
        }
    }
}
