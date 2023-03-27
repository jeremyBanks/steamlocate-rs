//! Encoding and decoding the binary VDF format used to store non-Steam game shortcuts.
//!
//! Based on the format descriptions at
//! <https://github.com/Corecii/steam-binary-vdf-ts#binary-vdf-format> and
//! <https://developer.valvesoftware.com/wiki/Add_Non-Steam_Game>.

use std::convert::TryInto;
use std::ffi::CString;

use indexmap::IndexMap;
use thiserror::Error;

use crate::Shortcut;

const TYPE_MAP: u8 = 0x00;
const TYPE_STR: u8 = 0x01;
const TYPE_INT: u8 = 0x02;
const TYPE_END: u8 = 0x08;

#[test]
fn test_round_trip_real_data() {
    use bstr::ByteSlice;

    let steam_dir = crate::SteamDir::locate().unwrap();

    let mut shortcut_data = Vec::new();

    let user_data = steam_dir.path.join("userdata");
    for entry in std::fs::read_dir(user_data)
        .ok()
        .unwrap()
        .filter_map(|e| e.ok())
    {
        let shortcuts_path = entry.path().join("config").join("shortcuts.vdf");
        if !shortcuts_path.is_file() {
            continue;
        }

        if let Ok(contents) = std::fs::read(&shortcuts_path) {
            shortcut_data.push(contents);
        }
    }

    for contents in &shortcut_data {
        let decoded = decode(&contents).unwrap();
        let encoded = encode(&decoded);
        assert_eq!(contents.as_bstr(), encoded.as_bstr());
    }
}

#[test]
fn test_really_add_something_to_your_library_for_real_maybe_remove_this_test() {
    let shortcut = Shortcut::new(
        "My Game".to_string(),
        "C:\\Program Files\\My Game\\MyGame.exe".to_string(),
    );
    shortcut.save_to_library(None);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Val {
    Map(Map),
    Str(CString),
    Int(Int),
}

pub type Map = IndexMap<CString, Val>;

pub type Int = u32;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("unexpected end of input")]
    UnexpectedEndOfInput,
    #[error("unexpected end of input")]
    InvalidMapItemPrefix,
}

fn decode_str(bytes: &mut &[u8]) -> Result<CString, DecodeError> {
    let mut len = 0;
    loop {
        let Some(&next) = bytes.get(len) else {
            return Err(DecodeError::UnexpectedEndOfInput);
        };
        if next == 0 {
            break;
        }
        len += 1;
    }
    let str = CString::new(&bytes[..len]).expect("unreachable");
    *bytes = &bytes[len + 1..];
    Ok(str)
}

fn decode_int(bytes: &mut &[u8]) -> Result<Int, DecodeError> {
    Ok({
        let int = Int::from_le_bytes(
            bytes[..4]
                .try_into()
                .map_err(|_| DecodeError::UnexpectedEndOfInput)?,
        );
        *bytes = &bytes[4..];
        int
    })
}

fn decode_map(mut bytes: &mut &[u8]) -> Result<Map, DecodeError> {
    Ok({
        let mut map = Map::new();

        while let Some(&next) = bytes.get(0) {
            *bytes = &bytes[1..];
            match next {
                TYPE_MAP => {
                    let key = decode_str(&mut bytes)?;
                    let value = decode_map(&mut bytes)?;
                    map.insert(key, Val::Map(value));
                }
                TYPE_STR => {
                    let key = decode_str(&mut bytes)?;
                    let value = decode_str(&mut bytes)?;
                    map.insert(key, Val::Str(value));
                }
                TYPE_INT => {
                    let key = decode_str(&mut bytes)?;
                    let value = decode_int(&mut bytes)?;
                    map.insert(key, Val::Int(value));
                }
                TYPE_END => break,
                _ => return Err(DecodeError::InvalidMapItemPrefix),
            }
        }

        map
    })
}

pub fn decode(mut bytes: &[u8]) -> Result<Map, DecodeError> {
    decode_map(&mut bytes)
}

pub fn encode(map: &Map) -> Vec<u8> {
    let mut bytes = Vec::new();

    for (key, value) in map {
        bytes.push(match value {
            Val::Map(_) => TYPE_MAP,
            Val::Str(_) => TYPE_STR,
            Val::Int(_) => TYPE_INT,
        });

        bytes.extend_from_slice(key.as_bytes_with_nul());

        match value {
            Val::Map(map) => bytes.extend_from_slice(&encode(map)),
            Val::Str(str) => bytes.extend_from_slice(str.as_bytes_with_nul()),
            Val::Int(int) => bytes.extend_from_slice(&int.to_le_bytes()),
        }
    }

    bytes.push(TYPE_END);

    bytes
}
