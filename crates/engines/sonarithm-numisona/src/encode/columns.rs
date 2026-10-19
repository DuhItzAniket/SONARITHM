// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Reading a field's values from its Arrow column.
//!
//! Numbers come from integer, floating-point, decimal, date, time-stamp and duration
//! columns, in the column's own unit (days for `Date32`, the time unit for time stamps and
//! durations). Text, boolean and dictionary columns give categories; a dictionary also gives
//! its declared category order (03 §3.1: ordinal fields are dictionaries with a declared
//! order). Nulls and NaN are missing values (03 §5).

use arrow_array::cast::AsArray;
use arrow_array::types::{
    Date32Type, Date64Type, DurationMicrosecondType, DurationMillisecondType,
    DurationNanosecondType, DurationSecondType, Float32Type, Float64Type, Int8Type, Int16Type,
    Int32Type, Int64Type, TimestampMicrosecondType, TimestampMillisecondType,
    TimestampNanosecondType, TimestampSecondType, UInt8Type, UInt16Type, UInt32Type, UInt64Type,
};
use arrow_array::{Array, ArrayRef, ArrowPrimitiveType, PrimitiveArray};
use arrow_schema::{DataType, TimeUnit};
use sonarithm_core::math;

use crate::scale::Category;

/// A field's values, one per row.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Column {
    /// Numbers; NaN marks a missing value.
    Numbers(Vec<f64>),
    /// Categories; `None` marks a missing value. `order` is a dictionary's declared order.
    Categories {
        values: Vec<Option<Category>>,
        order: Option<Vec<Category>>,
    },
}

#[cfg(test)]
impl Column {
    /// The number of rows.
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Numbers(values) => values.len(),
            Self::Categories { values, .. } => values.len(),
        }
    }
}

/// Reads `array`, or returns its type's name when no channel can read it.
pub(crate) fn read(array: &ArrayRef) -> Result<Column, String> {
    let data_type = array.data_type();
    if let Some(numbers) = numbers(array) {
        return Ok(Column::Numbers(numbers));
    }
    match data_type {
        DataType::Utf8 | DataType::LargeUtf8 | DataType::Utf8View | DataType::Boolean => {
            Ok(Column::Categories {
                values: texts(array.as_ref()).ok_or_else(|| data_type.to_string())?,
                order: None,
            })
        }
        DataType::Dictionary(_, value_type)
            if matches!(
                value_type.as_ref(),
                DataType::Utf8 | DataType::LargeUtf8 | DataType::Utf8View
            ) =>
        {
            let dictionary = array
                .as_any_dictionary_opt()
                .ok_or_else(|| data_type.to_string())?;
            let labels =
                texts(dictionary.values().as_ref()).ok_or_else(|| data_type.to_string())?;
            let keys = dictionary.normalized_keys();
            let values = (0..array.len())
                .map(|row| {
                    if array.is_null(row) {
                        None
                    } else {
                        keys.get(row)
                            .and_then(|&key| labels.get(key).cloned().flatten())
                    }
                })
                .collect();
            let order = labels.into_iter().flatten().collect();
            Ok(Column::Categories {
                values,
                order: Some(order),
            })
        }
        DataType::Null => Ok(Column::Numbers(vec![f64::NAN; array.len()])),
        other => Err(other.to_string()),
    }
}

/// Text labels of a string or boolean array.
fn texts(array: &dyn Array) -> Option<Vec<Option<Category>>> {
    let label = |text: &str| Some(Category::Text(text.to_owned()));
    let values = match array.data_type() {
        DataType::Utf8 => array
            .as_string_opt::<i32>()?
            .iter()
            .map(|v| v.and_then(label))
            .collect(),
        DataType::LargeUtf8 => array
            .as_string_opt::<i64>()?
            .iter()
            .map(|v| v.and_then(label))
            .collect(),
        DataType::Utf8View => array
            .as_string_view_opt()?
            .iter()
            .map(|v| v.and_then(label))
            .collect(),
        DataType::Boolean => array
            .as_boolean_opt()?
            .iter()
            .map(|v| v.and_then(|b| label(if b { "true" } else { "false" })))
            .collect(),
        _ => return None,
    };
    Some(values)
}

/// Numbers from a numeric, decimal or temporal array, or `None` for other types.
fn numbers(array: &ArrayRef) -> Option<Vec<f64>> {
    fn convert<T: ArrowPrimitiveType>(
        array: &dyn Array,
        to: impl Fn(T::Native) -> f64,
    ) -> Option<Vec<f64>> {
        let typed: &PrimitiveArray<T> = array.as_primitive_opt::<T>()?;
        Some(typed.iter().map(|v| v.map_or(f64::NAN, &to)).collect())
    }
    let a = array.as_ref();
    #[expect(
        clippy::cast_precision_loss,
        reason = "data values become f64 positions; integers above 2^53 lose their last bits, as in any f64 data path"
    )]
    match array.data_type() {
        DataType::Float64 => convert::<Float64Type>(a, |v| v),
        DataType::Float32 => convert::<Float32Type>(a, f64::from),
        DataType::Int8 => convert::<Int8Type>(a, f64::from),
        DataType::Int16 => convert::<Int16Type>(a, f64::from),
        DataType::Int32 => convert::<Int32Type>(a, f64::from),
        DataType::Int64 => convert::<Int64Type>(a, |v| v as f64),
        DataType::UInt8 => convert::<UInt8Type>(a, f64::from),
        DataType::UInt16 => convert::<UInt16Type>(a, f64::from),
        DataType::UInt32 => convert::<UInt32Type>(a, f64::from),
        DataType::UInt64 => convert::<UInt64Type>(a, |v| v as f64),
        DataType::Date32 => convert::<Date32Type>(a, f64::from),
        DataType::Date64 => convert::<Date64Type>(a, |v| v as f64),
        DataType::Timestamp(TimeUnit::Second, _) => convert::<TimestampSecondType>(a, |v| v as f64),
        DataType::Timestamp(TimeUnit::Millisecond, _) => {
            convert::<TimestampMillisecondType>(a, |v| v as f64)
        }
        DataType::Timestamp(TimeUnit::Microsecond, _) => {
            convert::<TimestampMicrosecondType>(a, |v| v as f64)
        }
        DataType::Timestamp(TimeUnit::Nanosecond, _) => {
            convert::<TimestampNanosecondType>(a, |v| v as f64)
        }
        DataType::Duration(TimeUnit::Second) => convert::<DurationSecondType>(a, |v| v as f64),
        DataType::Duration(TimeUnit::Millisecond) => {
            convert::<DurationMillisecondType>(a, |v| v as f64)
        }
        DataType::Duration(TimeUnit::Microsecond) => {
            convert::<DurationMicrosecondType>(a, |v| v as f64)
        }
        DataType::Duration(TimeUnit::Nanosecond) => {
            convert::<DurationNanosecondType>(a, |v| v as f64)
        }
        DataType::Decimal128(_, scale) => {
            let divisor = math::pow(10.0, f64::from(*scale));
            let typed = a.as_primitive_opt::<arrow_array::types::Decimal128Type>()?;
            Some(
                typed
                    .iter()
                    .map(|v| v.map_or(f64::NAN, |v| v as f64 / divisor))
                    .collect(),
            )
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::float_cmp,
        reason = "tests panic on the wrong variant and compare exact conversions"
    )]

    use std::sync::Arc;

    use arrow_array::{
        BooleanArray, Date32Array, DictionaryArray, Float64Array, Int32Array, StringArray,
        types::Int8Type as Key,
    };

    use super::*;

    fn text(s: &str) -> Category {
        Category::Text(s.to_owned())
    }

    #[test]
    fn numeric_and_temporal_columns_read_as_numbers() {
        let ints: ArrayRef = Arc::new(Int32Array::from(vec![Some(3), None, Some(-1)]));
        let Column::Numbers(values) = read(&ints).unwrap() else {
            panic!("numbers")
        };
        assert_eq!(values[0], 3.0);
        assert!(values[1].is_nan());
        let floats: ArrayRef = Arc::new(Float64Array::from(vec![0.5, f64::NAN]));
        assert_eq!(read(&floats).unwrap().len(), 2);
        let days: ArrayRef = Arc::new(Date32Array::from(vec![19_000]));
        assert_eq!(read(&days).unwrap(), Column::Numbers(vec![19_000.0]));
    }

    #[test]
    fn text_boolean_and_dictionary_columns_read_as_categories() {
        let strings: ArrayRef = Arc::new(StringArray::from(vec![Some("north"), None]));
        assert_eq!(
            read(&strings).unwrap(),
            Column::Categories {
                values: vec![Some(text("north")), None],
                order: None
            }
        );
        let flags: ArrayRef = Arc::new(BooleanArray::from(vec![true, false]));
        let Column::Categories { values, .. } = read(&flags).unwrap() else {
            panic!("categories")
        };
        assert_eq!(values, [Some(text("true")), Some(text("false"))]);
        let dictionary: DictionaryArray<Key> = vec!["low", "high", "low"].into_iter().collect();
        let dictionary: ArrayRef = Arc::new(dictionary);
        assert_eq!(
            read(&dictionary).unwrap(),
            Column::Categories {
                values: vec![Some(text("low")), Some(text("high")), Some(text("low"))],
                order: Some(vec![text("low"), text("high")]),
            }
        );
    }

    #[test]
    fn unsupported_columns_name_their_type() {
        let list: ArrayRef = Arc::new(arrow_array::ListArray::new_null(
            Arc::new(arrow_schema::Field::new("item", DataType::Int32, true)),
            1,
        ));
        assert!(read(&list).unwrap_err().contains("List"));
    }
}
