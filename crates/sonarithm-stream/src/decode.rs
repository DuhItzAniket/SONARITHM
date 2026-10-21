// SPDX-FileCopyrightText: 2026 SONARITHM contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Payload decoders: untrusted bytes in, one JSON object per message out, then a row that
//! fits a [`MessageSchema`]. They never panic and refuse oversized input; each has a fuzz
//! target (`crates/sonarithm-stream/fuzz`).
//!
//! | Transport | Payload | Object |
//! |---|---|---|
//! | WebSocket, MQTT | A JSON object (text) | As sent |
//! | OSC | An OSC 1.0 packet; bundles are flattened | `address`, then `v0`, `v1`, … for numbers, strings and booleans |
//! | Serial, file tail | One line: a JSON object, or numbers separated by commas, semicolons, tabs or spaces | As sent, or `v0`, `v1`, … |

use serde_json::{Map, Number, Value};

use crate::StreamError;
use crate::schema::{FieldKind, MessageSchema};

/// The largest message decoded, in bytes; larger ones are refused.
pub const MAX_MESSAGE: usize = 1 << 20;

/// The most messages one OSC packet (with nested bundles) may hold.
pub const MAX_OSC_MESSAGES: usize = 1024;

/// One value of a row.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Cell {
    /// A finite number.
    Number(f64),
    /// Text.
    Text(String),
    /// A boolean.
    Bool(bool),
    /// No value.
    Null,
}

fn decode_error(why: impl Into<String>) -> StreamError {
    StreamError::Decode(why.into())
}

fn check_size(bytes: &[u8]) -> Result<(), StreamError> {
    if bytes.len() > MAX_MESSAGE {
        return Err(decode_error(format!(
            "a message of {} bytes is over the {MAX_MESSAGE}-byte limit",
            bytes.len()
        )));
    }
    Ok(())
}

/// A JSON object message.
///
/// # Errors
///
/// [`StreamError::Decode`] for oversized input, invalid JSON or JSON that is not an object.
pub fn json_object(bytes: &[u8]) -> Result<Map<String, Value>, StreamError> {
    check_size(bytes)?;
    match serde_json::from_slice::<Value>(bytes) {
        Ok(Value::Object(map)) => Ok(map),
        Ok(_) => Err(decode_error("the message is JSON but not an object")),
        Err(err) => Err(decode_error(format!("the message is not JSON: {err}"))),
    }
}

/// One line from a serial port or a followed file.
///
/// # Errors
///
/// [`StreamError::Decode`] for an oversized or empty line, a line that is neither a JSON
/// object nor separated numbers, or a number that is not finite.
pub fn line(text: &str) -> Result<Map<String, Value>, StreamError> {
    check_size(text.as_bytes())?;
    let text = text.trim();
    if text.is_empty() {
        return Err(decode_error("the line is empty"));
    }
    if text.starts_with('{') {
        return json_object(text.as_bytes());
    }
    let mut object = Map::new();
    let parts = text
        .split([',', ';', '\t', ' '])
        .filter(|part| !part.is_empty());
    for (index, part) in parts.enumerate() {
        let number: f64 = part
            .parse()
            .map_err(|_| decode_error(format!("`{part}` is not a number")))?;
        let number = Number::from_f64(number)
            .ok_or_else(|| decode_error(format!("`{part}` is not a finite number")))?;
        object.insert(format!("v{index}"), Value::Number(number));
    }
    Ok(object)
}

/// The messages of one OSC packet.
///
/// # Errors
///
/// [`StreamError::Decode`] for oversized input, a packet that is not OSC 1.0, or a
/// packet with more than [`MAX_OSC_MESSAGES`] messages.
#[cfg(feature = "osc")]
pub fn osc(bytes: &[u8]) -> Result<Vec<Map<String, Value>>, StreamError> {
    use rosc::{OscPacket, OscType};

    check_size(bytes)?;
    let (_, packet) = rosc::decoder::decode_udp(bytes)
        .map_err(|err| decode_error(format!("the packet is not OSC: {err:?}")))?;
    let mut messages = Vec::new();
    let mut pending = vec![packet];
    while let Some(packet) = pending.pop() {
        match packet {
            OscPacket::Message(message) => {
                if messages.len() >= MAX_OSC_MESSAGES {
                    return Err(decode_error("the packet holds too many messages"));
                }
                let mut object = Map::new();
                object.insert("address".into(), Value::String(message.addr));
                let mut index = 0;
                for arg in message.args {
                    let value = match arg {
                        OscType::Int(v) => Number::from_f64(f64::from(v)).map(Value::Number),
                        #[expect(clippy::cast_precision_loss, reason = "OSC longs as floats")]
                        OscType::Long(v) => Number::from_f64(v as f64).map(Value::Number),
                        OscType::Float(v) => Number::from_f64(f64::from(v)).map(Value::Number),
                        OscType::Double(v) => Number::from_f64(v).map(Value::Number),
                        OscType::String(v) => Some(Value::String(v)),
                        OscType::Bool(v) => Some(Value::Bool(v)),
                        _ => None,
                    };
                    if let Some(value) = value {
                        object.insert(format!("v{index}"), value);
                        index += 1;
                    }
                }
                messages.push(object);
            }
            OscPacket::Bundle(bundle) => {
                if pending.len() + bundle.content.len() > MAX_OSC_MESSAGES {
                    return Err(decode_error("the packet holds too many messages"));
                }
                // Reversed so messages keep their order when popped.
                pending.extend(bundle.content.into_iter().rev());
            }
        }
    }
    Ok(messages)
}

/// The row of `message` for `schema`, cells in the schema's field order. Fields the
/// message lacks are null; extra keys are ignored.
///
/// # Errors
///
/// [`StreamError::Decode`] when a required field is missing or null, or a value has
/// another type than its field.
pub fn row(message: &Map<String, Value>, schema: &MessageSchema) -> Result<Vec<Cell>, StreamError> {
    schema
        .fields()
        .iter()
        .map(|(name, kind, required)| match (message.get(name), kind) {
            (None | Some(Value::Null), _) if *required => Err(decode_error(format!(
                "the required field `{name}` is missing"
            ))),
            (None | Some(Value::Null), _) => Ok(Cell::Null),
            (Some(Value::Number(n)), FieldKind::Number) => n
                .as_f64()
                .filter(|v| v.is_finite())
                .map(Cell::Number)
                .ok_or_else(|| decode_error(format!("`{name}` is not a finite number"))),
            (Some(Value::String(s)), FieldKind::Text) => Ok(Cell::Text(s.clone())),
            (Some(Value::Bool(b)), FieldKind::Boolean) => Ok(Cell::Bool(*b)),
            (Some(_), _) => Err(decode_error(format!(
                "`{name}` has the wrong type for the schema"
            ))),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn schema() -> MessageSchema {
        MessageSchema::from_json_schema(&json!({
            "type": "object",
            "properties": { "t": { "type": "number" }, "host": { "type": "string" }, "up": { "type": "boolean" } },
            "required": ["t"]
        }))
        .unwrap()
    }

    #[test]
    fn json_messages_become_rows_in_field_order() {
        let message = json_object(br#"{"up": true, "t": 2.5, "extra": [1]}"#).unwrap();
        let row = row(&message, &schema()).unwrap();
        assert_eq!(row, [Cell::Null, Cell::Number(2.5), Cell::Bool(true)]);
    }

    #[test]
    fn invalid_messages_are_refused() {
        assert!(json_object(b"[1, 2]").is_err());
        assert!(json_object(b"{ nope").is_err());
        assert!(json_object(&vec![b' '; MAX_MESSAGE + 1]).is_err());
        let missing = json_object(br#"{"host": "a"}"#).unwrap();
        assert!(row(&missing, &schema()).is_err());
        let wrong = json_object(br#"{"t": "soon"}"#).unwrap();
        assert!(row(&wrong, &schema()).is_err());
    }

    #[test]
    fn lines_are_json_or_separated_numbers() {
        assert_eq!(line("1.5, 2;3\t4 5").unwrap().len(), 5);
        assert_eq!(line(r#"{"t": 1}"#).unwrap()["t"], json!(1));
        assert!(line("  ").is_err());
        assert!(line("1, two").is_err());
        assert!(line("1e999").is_err());
    }

    #[cfg(feature = "osc")]
    #[test]
    fn osc_bundles_flatten_in_order() {
        use rosc::{OscBundle, OscMessage, OscPacket, OscTime, OscType};
        let message = |addr: &str, v: f32| {
            OscPacket::Message(OscMessage {
                addr: addr.to_owned(),
                args: vec![OscType::Float(v), OscType::String("x".into())],
            })
        };
        let packet = OscPacket::Bundle(OscBundle {
            timetag: OscTime::from((0, 1)),
            content: vec![message("/a", 1.0), message("/b", 2.0)],
        });
        let bytes = rosc::encoder::encode(&packet).unwrap();
        let objects = osc(&bytes).unwrap();
        assert_eq!(objects.len(), 2);
        assert_eq!(objects[0]["address"], "/a");
        assert_eq!(objects[1]["v0"], json!(2.0));
        assert_eq!(objects[1]["v1"], "x");
        assert!(osc(b"not osc").is_err());
    }
}
