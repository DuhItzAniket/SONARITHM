// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Reading a field's values for navigation.
//!
//! Numbers come from integer, floating-point, decimal and temporal columns. Dates become
//! days since 1970-01-01 and time stamps seconds since the epoch, so the navigation tree
//! can say what an x value is ([`XType`]). Text, boolean and text-dictionary columns give
//! categories. Nulls and NaN are missing values (03 §5).

use arrow_array::cast::AsArray;
use arrow_array::types::{
    Date32Type, Date64Type, Decimal128Type, Float32Type, Float64Type, Int8Type, Int16Type,
    Int32Type, Int64Type, TimestampMicrosecondType, TimestampMillisecondType,
    TimestampNanosecondType, TimestampSecondType, UInt8Type, UInt16Type, UInt32Type, UInt64Type,
};
use arrow_array::{Array, ArrayRef, ArrowPrimitiveType, PrimitiveArray};
use arrow_schema::{DataType, TimeUnit};
use sonarithm_core::math;
use sonarithm_core::nav::XType;

/// A field's values, one per row.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Values {
    /// Numbers (NaN for missing) and what they are.
    Numbers(Vec<f64>, XType),
    /// Category labels (`None` for missing).
    Categories(Vec<Option<String>>),
}

/// Reads `array`, or `None` for a type navigation cannot use.
pub(crate) fn read(array: &ArrayRef) -> Option<Values> {
    if let Some((numbers, kind)) = numbers(array) {
        return Some(Values::Numbers(numbers, kind));
    }
    let labels = match array.data_type() {
        DataType::Dictionary(_, _) => {
            let dictionary = array.as_any_dictionary_opt()?;
            let labels = texts(dictionary.values().as_ref())?;
            let keys = dictionary.normalized_keys();
            (0..array.len())
                .map(|row| {
                    if array.is_null(row) {
                        None
                    } else {
                        keys.get(row)
                            .and_then(|&key| labels.get(key).cloned().flatten())
                    }
                })
                .collect()
        }
        _ => texts(array.as_ref())?,
    };
    Some(Values::Categories(labels))
}

/// Text labels of a string or boolean array.
fn texts(array: &dyn Array) -> Option<Vec<Option<String>>> {
    let owned = |v: Option<&str>| v.map(str::to_owned);
    Some(match array.data_type() {
        DataType::Utf8 => array.as_string_opt::<i32>()?.iter().map(owned).collect(),
        DataType::LargeUtf8 => array.as_string_opt::<i64>()?.iter().map(owned).collect(),
        DataType::Utf8View => array.as_string_view_opt()?.iter().map(owned).collect(),
        DataType::Boolean => array
            .as_boolean_opt()?
            .iter()
            .map(|v| v.map(|b| if b { "true" } else { "false" }.to_owned()))
            .collect(),
        _ => return None,
    })
}

/// Numbers from a numeric, decimal or temporal array, converted to the unit [`XType`]
/// names, or `None` for other types.
fn numbers(array: &ArrayRef) -> Option<(Vec<f64>, XType)> {
    fn convert<T: ArrowPrimitiveType>(
        array: &dyn Array,
        to: impl Fn(T::Native) -> f64,
    ) -> Option<Vec<f64>> {
        let typed: &PrimitiveArray<T> = array.as_primitive_opt::<T>()?;
        Some(typed.iter().map(|v| v.map_or(f64::NAN, &to)).collect())
    }
    let a = array.as_ref();
    let number = |values: Option<Vec<f64>>| values.map(|v| (v, XType::Number));
    let instant = |values: Option<Vec<f64>>| values.map(|v| (v, XType::Timestamp));
    #[expect(
        clippy::cast_precision_loss,
        reason = "data values become f64 positions; integers above 2^53 lose their last bits, as in any f64 data path"
    )]
    match array.data_type() {
        DataType::Float64 => number(convert::<Float64Type>(a, |v| v)),
        DataType::Float32 => number(convert::<Float32Type>(a, f64::from)),
        DataType::Int8 => number(convert::<Int8Type>(a, f64::from)),
        DataType::Int16 => number(convert::<Int16Type>(a, f64::from)),
        DataType::Int32 => number(convert::<Int32Type>(a, f64::from)),
        DataType::Int64 => number(convert::<Int64Type>(a, |v| v as f64)),
        DataType::UInt8 => number(convert::<UInt8Type>(a, f64::from)),
        DataType::UInt16 => number(convert::<UInt16Type>(a, f64::from)),
        DataType::UInt32 => number(convert::<UInt32Type>(a, f64::from)),
        DataType::UInt64 => number(convert::<UInt64Type>(a, |v| v as f64)),
        DataType::Decimal128(_, scale) => {
            let divisor = math::pow(10.0, f64::from(*scale));
            number(convert::<Decimal128Type>(a, |v| v as f64 / divisor))
        }
        DataType::Date32 => convert::<Date32Type>(a, f64::from).map(|v| (v, XType::Date)),
        DataType::Date64 => instant(convert::<Date64Type>(a, |v| v as f64 / 1e3)),
        DataType::Timestamp(TimeUnit::Second, _) => {
            instant(convert::<TimestampSecondType>(a, |v| v as f64))
        }
        DataType::Timestamp(TimeUnit::Millisecond, _) => {
            instant(convert::<TimestampMillisecondType>(a, |v| v as f64 / 1e3))
        }
        DataType::Timestamp(TimeUnit::Microsecond, _) => {
            instant(convert::<TimestampMicrosecondType>(a, |v| v as f64 / 1e6))
        }
        DataType::Timestamp(TimeUnit::Nanosecond, _) => {
            instant(convert::<TimestampNanosecondType>(a, |v| v as f64 / 1e9))
        }
        DataType::Null => Some((vec![f64::NAN; array.len()], XType::Number)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::float_cmp,
        clippy::panic,
        reason = "the conversions under test are exact for these inputs; tests panic on the wrong variant"
    )]

    use std::sync::Arc;

    use arrow_array::{
        BooleanArray, Date32Array, DictionaryArray, Float64Array, StringArray,
        TimestampMillisecondArray, types::Int8Type as Key,
    };

    use super::*;

    #[test]
    fn numbers_and_temporal_columns_read_with_their_type() {
        let floats: ArrayRef = Arc::new(Float64Array::from(vec![Some(1.5), None]));
        let Some(Values::Numbers(values, XType::Number)) = read(&floats) else {
            panic!("floats are numbers");
        };
        assert_eq!(values[0], 1.5);
        assert!(values[1].is_nan());

        let dates: ArrayRef = Arc::new(Date32Array::from(vec![20_648]));
        assert_eq!(
            read(&dates),
            Some(Values::Numbers(vec![20_648.0], XType::Date))
        );

        let stamps: ArrayRef = Arc::new(TimestampMillisecondArray::from(vec![1_500]));
        assert_eq!(
            read(&stamps),
            Some(Values::Numbers(vec![1.5], XType::Timestamp))
        );
    }

    #[test]
    fn text_boolean_and_dictionary_columns_read_as_categories() {
        let text: ArrayRef = Arc::new(StringArray::from(vec![Some("apple"), None]));
        assert_eq!(
            read(&text),
            Some(Values::Categories(vec![Some("apple".to_owned()), None]))
        );
        let flags: ArrayRef = Arc::new(BooleanArray::from(vec![true]));
        assert_eq!(
            read(&flags),
            Some(Values::Categories(vec![Some("true".to_owned())]))
        );
        let dictionary: DictionaryArray<Key> = vec!["low", "high", "low"].into_iter().collect();
        let dictionary: ArrayRef = Arc::new(dictionary);
        assert_eq!(
            read(&dictionary),
            Some(Values::Categories(vec![
                Some("low".to_owned()),
                Some("high".to_owned()),
                Some("low".to_owned())
            ]))
        );
    }
}
