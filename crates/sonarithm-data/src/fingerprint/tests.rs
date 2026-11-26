// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! RFC-0008 §7: layout independence, sensitivity, and the encoding itself.

use std::collections::HashMap;

use arrow_array::{
    Float64Array, Int32Array, Int64Array, IntervalYearMonthArray,
    LargeListArray, ListArray, RunArray, StringViewArray,
};
use proptest::prelude::*;
use sonarithm_core::TableId;

use super::*;
use crate::meta::{FieldMeta, FieldType};

fn table(values: ArrayRef, meta: &FieldMeta) -> Table {
    let schema = Arc::new(Schema::new(vec![meta.apply(Field::new(
        "v",
        values.data_type().clone(),
        true,
    ))]));
    let batch = RecordBatch::try_new(schema, vec![values]).unwrap();
    Table::from_batch(TableId(0), "t", batch)
}

fn fingerprint(values: ArrayRef) -> Sha256Digest {
    table_fingerprint(&table(values, &FieldMeta::new())).unwrap()
}

fn batches(schema: Arc<Schema>, parts: Vec<ArrayRef>) -> Table {
    let batches = parts
        .into_iter()
        .map(|part| RecordBatch::try_new(Arc::clone(&schema), vec![part]).unwrap())
        .collect();
    Table::new(TableId(1), "split", schema, batches).unwrap()
}

#[test]
fn batching_slicing_and_bytes_under_nulls_do_not_matter() {
    let meta = FieldMeta::new();
    let whole = table(
        Arc::new(Int64Array::from(vec![Some(1), None, Some(3)])),
        &meta,
    );
    // The same values, built from a slice of a longer array with garbage under the null.
    let longer = Int64Array::from(vec![Some(9), Some(1), Some(42), Some(3)]);
    let mask = BooleanArray::from(vec![false, false, true, false]);
    let with_garbage = arrow_select::nullif::nullif(&longer, &mask)
        .unwrap()
        .slice(1, 3);
    assert!(with_garbage.is_null(1));
    let sliced = table(with_garbage, &meta);
    assert_eq!(
        table_fingerprint(&whole).unwrap(),
        table_fingerprint(&sliced).unwrap()
    );
    // Two batches instead of one, under another name and ID.
    let split = Table::new(
        TableId(5),
        "other name",
        Arc::clone(whole.schema()),
        vec![
            whole.batches()[0].slice(0, 1),
            whole.batches()[0].slice(1, 2),
        ],
    )
    .unwrap();
    assert_eq!(
        table_fingerprint(&whole).unwrap(),
        table_fingerprint(&split).unwrap()
    );
}

#[test]
fn values_types_and_snr_metadata_change_the_fingerprint() {
    let base = table(
        Arc::new(Float64Array::from(vec![1.0, 2.0])),
        &FieldMeta::new(),
    );
    let other_value = table(
        Arc::new(Float64Array::from(vec![1.0, 2.5])),
        &FieldMeta::new(),
    );
    let other_type = table(Arc::new(Int64Array::from(vec![1, 2])), &FieldMeta::new());
    let other_meta = table(
        Arc::new(Float64Array::from(vec![1.0, 2.0])),
        &FieldMeta::new().with_type(FieldType::Quantitative),
    );
    let other_null = table(
        Arc::new(Float64Array::from(vec![Some(1.0), None])),
        &FieldMeta::new(),
    );
    let base_fp = table_fingerprint(&base).unwrap();
    for other in [&other_value, &other_type, &other_meta, &other_null] {
        assert_ne!(table_fingerprint(other).unwrap(), base_fp);
    }
    assert_eq!(
        table_fingerprint(&base).unwrap(),
        base_fp,
        "stable across calls"
    );
}

#[test]
fn a_field_name_and_a_schema_snr_key_change_the_fingerprint() {
    let values: ArrayRef = Arc::new(Int64Array::from(vec![1, 2]));
    let named = |name: &str, meta: HashMap<String, String>| {
        let schema = Schema::new(vec![Field::new(name, DataType::Int64, true)]).with_metadata(meta);
        let batch = RecordBatch::try_new(Arc::new(schema), vec![Arc::clone(&values)]).unwrap();
        table_fingerprint(&Table::from_batch(TableId(0), "t", batch)).unwrap()
    };
    let base = named("v", HashMap::new());
    assert_ne!(named("w", HashMap::new()), base);
    let snr = HashMap::from([("snr.function.expr".to_owned(), "x^2".to_owned())]);
    assert_ne!(named("v", snr), base, "schema-level snr.* keys count (CURVASONA)");
}

#[test]
fn nullability_and_other_metadata_do_not_change_the_fingerprint() {
    let values: ArrayRef = Arc::new(Int64Array::from(vec![1, 2]));
    let build = |field: Field, schema_meta: HashMap<String, String>| {
        let schema = Schema::new(vec![field]).with_metadata(schema_meta);
        let batch = RecordBatch::try_new(Arc::new(schema), vec![Arc::clone(&values)]).unwrap();
        table_fingerprint(&Table::from_batch(TableId(0), "t", batch)).unwrap()
    };
    let base = build(Field::new("v", DataType::Int64, true), HashMap::new());
    let not_nullable = build(Field::new("v", DataType::Int64, false), HashMap::new());
    let pandas = HashMap::from([("pandas".to_owned(), "{\"index_columns\":[]}".to_owned())]);
    let library = build(
        Field::new("v", DataType::Int64, true)
            .with_metadata(HashMap::from([("ARROW:extension:name".to_owned(), "x".to_owned())])),
        pandas,
    );
    assert_eq!(not_nullable, base);
    assert_eq!(library, base);
}

#[test]
fn every_string_layout_fingerprints_alike() {
    let values = vec![Some("north"), None, Some(""), Some("a longer string than twelve bytes")];
    let plain = fingerprint(Arc::new(StringArray::from(values.clone())));
    let large = fingerprint(Arc::new(LargeStringArray::from(values.clone())));
    let view = fingerprint(Arc::new(StringViewArray::from(values)));
    assert_eq!(large, plain);
    assert_eq!(view, plain);
}

#[test]
fn dictionary_key_width_does_not_matter_but_the_dictionary_does() {
    let narrow: DictionaryArray<UInt8Type> = vec!["low", "high", "low"].into_iter().collect();
    let wide: DictionaryArray<Int32Type> = vec!["low", "high", "low"].into_iter().collect();
    assert_eq!(fingerprint(Arc::new(narrow)), fingerprint(Arc::new(wide)));

    // A plain column with the same strings is a different column: NUMISONA takes a
    // dictionary's order as the category order.
    let dictionary: DictionaryArray<Int32Type> = vec!["low", "high", "low"].into_iter().collect();
    let plain = StringArray::from(vec!["low", "high", "low"]);
    assert_ne!(
        fingerprint(Arc::new(dictionary.clone())),
        fingerprint(Arc::new(plain))
    );

    // The same rows over a dictionary in another order, or with an unused value.
    let reordered = DictionaryArray::<Int32Type>::try_new(
        Int32Array::from(vec![1, 0, 1]),
        Arc::new(StringArray::from(vec!["high", "low"])),
    )
    .unwrap();
    let unused = DictionaryArray::<Int32Type>::try_new(
        Int32Array::from(vec![0, 1, 0]),
        Arc::new(StringArray::from(vec!["low", "high", "medium"])),
    )
    .unwrap();
    let base = fingerprint(Arc::new(dictionary));
    assert_ne!(fingerprint(Arc::new(reordered)), base);
    assert_ne!(fingerprint(Arc::new(unused)), base);
}

#[test]
fn batches_sharing_a_dictionary_fingerprint_as_one() {
    let whole = DictionaryArray::<Int32Type>::try_new(
        Int32Array::from(vec![0, 1, 0, 1]),
        Arc::new(StringArray::from(vec!["low", "high"])),
    )
    .unwrap();
    let whole: ArrayRef = Arc::new(whole);
    let one = table(Arc::clone(&whole), &FieldMeta::new());
    let split = batches(
        Arc::clone(one.schema()),
        vec![whole.slice(0, 1), whole.slice(1, 3)],
    );
    assert_eq!(
        table_fingerprint(&split).unwrap(),
        table_fingerprint(&one).unwrap()
    );
}

#[test]
fn floats_are_hashed_bit_exact() {
    let zero = fingerprint(Arc::new(Float64Array::from(vec![0.0])));
    let negative_zero = fingerprint(Arc::new(Float64Array::from(vec![-0.0])));
    assert_ne!(zero, negative_zero, "-0.0 and 0.0 differ (RFC-0008 §4)");
    let nan = fingerprint(Arc::new(Float64Array::from(vec![f64::NAN])));
    let other_nan = fingerprint(Arc::new(Float64Array::from(vec![f64::from_bits(
        f64::NAN.to_bits() | 1,
    )])));
    assert_ne!(nan, other_nan, "NaN payloads differ");
}

#[test]
fn list_layouts_fingerprint_alike() {
    let rows = vec![Some(vec![Some(1), None]), None, Some(vec![])];
    let list = ListArray::from_iter_primitive::<Int64Type, _, _>(rows.clone());
    let large = LargeListArray::from_iter_primitive::<Int64Type, _, _>(rows);
    assert_eq!(fingerprint(Arc::new(list)), fingerprint(Arc::new(large)));
}

/// `values` run-end encoded, one run per value, with run ends of type `R`.
fn run_encoded<R: arrow_array::types::RunEndIndexType>(
    ends: PrimitiveArray<R>,
    values: Vec<Option<f64>>,
) -> ArrayRef {
    Arc::new(RunArray::<R>::try_new(&ends, &Float64Array::from(values)).unwrap())
}

#[test]
fn run_end_encoding_is_layout() {
    let values = vec![Some(1.5), None, Some(2.5)];
    let plain = fingerprint(Arc::new(Float64Array::from(values.clone())));
    let narrow = run_encoded(Int32Array::from(vec![1, 2, 3]), values.clone());
    let wide = run_encoded(Int64Array::from(vec![1, 2, 3]), values);
    assert_eq!(fingerprint(narrow), plain);
    assert_eq!(fingerprint(wide), plain);
}

#[test]
fn an_unsupported_type_is_refused_with_its_code() {
    let intervals = IntervalYearMonthArray::from(vec![14]);
    let err = table_fingerprint(&table(Arc::new(intervals), &FieldMeta::new())).unwrap_err();
    assert!(matches!(err, DataError::Unfingerprintable { .. }));
    assert_eq!(err.to_diagnostic().code.as_str(), "SNR-DATA-007");
}

/// The encoding of RFC-0008 §2 and §3, written out by hand for a two-column table: the
/// implementation must produce exactly this input.
#[test]
fn the_input_is_the_encoding_rfc_0008_defines() {
    let schema = Arc::new(Schema::new(vec![
        Field::new("n", DataType::Int64, true),
        Field::new("s", DataType::Utf8, true).with_metadata(HashMap::from([
            ("snr.type".to_owned(), "nominal".to_owned()),
            ("pandas".to_owned(), "ignored".to_owned()),
        ])),
    ]));
    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int64Array::from(vec![Some(7), None])),
            Arc::new(StringArray::from(vec![Some("é"), Some("")])),
        ],
    )
    .unwrap();
    let got = table_fingerprint(&Table::from_batch(TableId(0), "t", batch)).unwrap();

    let header = r#"{"fields":[{"meta":{},"name":"n","type":"int64"},{"meta":{"snr.type":"nominal"},"name":"s","type":"string"}],"meta":{}}"#;
    let mut input = b"sonarithm-data table fingerprint v2\0".to_vec();
    input.extend_from_slice(&(header.len() as u64).to_le_bytes());
    input.extend_from_slice(header.as_bytes());
    input.extend_from_slice(&2_u64.to_le_bytes());
    // n: [1, 7i64], [0]
    input.extend_from_slice(&10_u64.to_le_bytes());
    input.push(1);
    input.extend_from_slice(&7_i64.to_le_bytes());
    input.push(0);
    // s: [1, len 2, "é"], [1, len 0]
    input.extend_from_slice(&(1 + 8 + 2 + 1 + 8_u64).to_le_bytes());
    input.push(1);
    input.extend_from_slice(&2_u64.to_le_bytes());
    input.extend_from_slice("é".as_bytes());
    input.push(1);
    input.extend_from_slice(&0_u64.to_le_bytes());
    assert_eq!(got, Sha256Digest::of(&input));
}

#[test]
fn json_strings_are_escaped_as_rfc_8785_writes_them() {
    let mut out = String::new();
    json_string(&mut out, "a\"b\\c\u{8}\u{c}\n\r\t\u{1}\u{1f}é€😀/");
    assert_eq!(out, r#""a\"b\\c\b\f\n\r\t\u0001\u001fé€😀/""#);
}

#[test]
fn snr_keys_sort_by_utf16_code_units() {
    // U+FF5E sorts after U+1F600 in UTF-8 byte order but before it in UTF-16.
    let meta = HashMap::from([
        ("snr.\u{1F600}".to_owned(), "a".to_owned()),
        ("snr.\u{FF5E}".to_owned(), "b".to_owned()),
    ]);
    let mut out = String::new();
    snr_meta(&mut out, meta.iter());
    assert_eq!(out, "{\"snr.\u{1F600}\":\"a\",\"snr.\u{FF5E}\":\"b\"}");
}

/// Builds `values` as one string column in `layout` (0 plain, 1 large, 2 view).
fn strings(values: &[Option<String>], layout: u8) -> ArrayRef {
    match layout {
        0 => Arc::new(StringArray::from(values.to_vec())),
        1 => Arc::new(LargeStringArray::from(values.to_vec())),
        _ => Arc::new(StringViewArray::from(values.to_vec())),
    }
}

/// Splits `0..len` at the given cut points.
fn cuts(len: usize, mut at: Vec<usize>) -> Vec<(usize, usize)> {
    at.retain(|&cut| cut > 0 && cut < len);
    at.sort_unstable();
    at.dedup();
    let mut bounds = vec![0];
    bounds.extend(at);
    bounds.push(len);
    bounds.windows(2).map(|w| (w[0], w[1] - w[0])).collect()
}

proptest! {
    /// Layout independence: batching, string layout and dictionary key width never move
    /// the fingerprint.
    #[test]
    fn layout_never_changes_the_fingerprint(
        numbers in proptest::collection::vec(proptest::option::of(any::<f64>()), 1..40),
        texts in proptest::collection::vec(proptest::option::of("[a-c]{0,3}"), 40),
        at in proptest::collection::vec(0_usize..40, 0..4),
        layout in 0_u8..3,
        wide_keys in any::<bool>(),
    ) {
        let len = numbers.len();
        let texts = &texts[..len];
        let schema = |text: DataType| Arc::new(Schema::new(vec![
            Field::new("x", DataType::Float64, true),
            Field::new("s", text, true),
            Field::new("d", DataType::Dictionary(
                Box::new(if wide_keys { DataType::Int32 } else { DataType::UInt8 }),
                Box::new(DataType::Utf8),
            ), true),
        ]));
        let dictionary = |rows: &[Option<String>]| -> ArrayRef {
            let keys: Vec<Option<i32>> = rows
                .iter()
                .map(|row| row.as_ref().map(|t| i32::try_from(t.len()).unwrap()))
                .collect();
            let values: ArrayRef = Arc::new(StringArray::from(vec!["", "a", "bb", "ccc"]));
            if wide_keys {
                Arc::new(DictionaryArray::<Int32Type>::try_new(Int32Array::from(keys), values).unwrap())
            } else {
                let keys: Vec<Option<u8>> = keys.iter().map(|k| k.map(|k| u8::try_from(k).unwrap())).collect();
                Arc::new(DictionaryArray::<UInt8Type>::try_new(keys.into(), values).unwrap())
            }
        };
        let canonical_dictionary = |rows: &[Option<String>]| -> ArrayRef {
            let keys: Vec<Option<u8>> = rows.iter().map(|row| row.as_ref().map(|t| u8::try_from(t.len()).unwrap())).collect();
            let values: ArrayRef = Arc::new(StringArray::from(vec!["", "a", "bb", "ccc"]));
            Arc::new(DictionaryArray::<UInt8Type>::try_new(keys.into(), values).unwrap())
        };
        let canonical_schema = Arc::new(Schema::new(vec![
            Field::new("x", DataType::Float64, true),
            Field::new("s", DataType::Utf8, true),
            Field::new("d", DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)), true),
        ]));
        let one = RecordBatch::try_new(Arc::clone(&canonical_schema), vec![
            Arc::new(Float64Array::from(numbers.clone())),
            strings(texts, 0),
            canonical_dictionary(texts),
        ]).unwrap();
        let expected = table_fingerprint(&Table::from_batch(TableId(0), "t", one)).unwrap();

        let laid_out = schema(strings(&[], layout).data_type().clone());
        let parts = cuts(len, at)
            .into_iter()
            .map(|(start, n)| RecordBatch::try_new(Arc::clone(&laid_out), vec![
                Arc::new(Float64Array::from(numbers[start..start + n].to_vec())),
                strings(&texts[start..start + n], layout),
                dictionary(&texts[start..start + n]),
            ]).unwrap())
            .collect();
        let split = Table::new(TableId(9), "split", laid_out, parts).unwrap();
        prop_assert_eq!(table_fingerprint(&split).unwrap(), expected);
    }

    /// Sensitivity: changing any one value, including only its float bits, moves it.
    #[test]
    fn any_value_change_moves_the_fingerprint(
        numbers in proptest::collection::vec(any::<u64>(), 1..40),
        index in any::<prop::sample::Index>(),
        flip in 0_u32..64,
    ) {
        let floats = |bits: &[u64]| -> ArrayRef {
            Arc::new(Float64Array::from(bits.iter().map(|b| f64::from_bits(*b)).collect::<Vec<_>>()))
        };
        let mut changed = numbers.clone();
        let at = index.index(changed.len());
        changed[at] ^= 1 << flip;
        prop_assert_ne!(fingerprint(floats(&numbers)), fingerprint(floats(&changed)));
        let mut nulled: Vec<Option<f64>> = numbers.iter().map(|b| Some(f64::from_bits(*b))).collect();
        nulled[at] = None;
        prop_assert_ne!(fingerprint(floats(&numbers)), fingerprint(Arc::new(Float64Array::from(nulled))));
    }
}
