use derive_more::From;
use derive_more::TryInto;
use thiserror::Error;

const TYPE_MAP: u8 = 0x00;
const TYPE_STR: u8 = 0x01;
const TYPE_INT: u8 = 0x02;
const TYPE_END: u8 = 0x08;

#[cfg(test)]
#[test]
fn test_shortcuts() {
    fn shortcuts() -> Vec<Map> {
        let mut steam_dir = steamlocate::SteamDir::locate().unwrap();

        let mut shortcuts = Vec::new();

        // Find and parse each `userdata/<user_id>/config/shortcuts.vdf` file
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
                let map = decode(&mut &contents[..]).unwrap();
                shortcuts.push(map);
            }
        }

        shortcuts
    }

    let shortcuts = shortcuts();
    panic!("{shortcuts:#?}");
}

#[derive(Debug, Clone, PartialEq, Eq, From, TryInto)]
#[repr(u8)]
pub enum Val {
    Map(Map) = TYPE_MAP,
    Str(Str) = TYPE_STR,
    Int(Int) = TYPE_INT,
    End = TYPE_END,
}

pub type Map = indexmap::IndexMap<Str, Val>;
pub type Str = std::ffi::CString;
pub type Int = u32;

#[derive(Debug, Error)]
#[error("unknown error while decoding VDF")]
pub struct DecodeError;

fn decode_str(bytes: &mut &[u8]) -> Result<Str, DecodeError> {
    Ok({
        let mut len = 0;
        while let Some(&next) = bytes.get(len) {
            if next == 0 {
                break;
            }
            len += 1;
        }

        let str = Str::new(&bytes[..len]).map_err(|_| DecodeError)?;
        *bytes = &bytes[len + 1..];
        str
    })
}

fn decode_int(bytes: &mut &[u8]) -> Result<Int, DecodeError> {
    Ok({
        let int = Int::from_le_bytes(bytes[..4].try_into().map_err(|_| DecodeError)?);
        *bytes = &bytes[4..];
        int
    })
}

pub fn decode(mut bytes: &mut &[u8]) -> Result<Map, DecodeError> {
    Ok({
        let mut map = Map::new();

        while let Some(&next) = bytes.get(0) {
            *bytes = &bytes[1..];
            match next {
                TYPE_MAP => {
                    let key = decode_str(&mut bytes)?;
                    let value = decode(&mut bytes)?;
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
                _ => return Err(DecodeError),
            }
        }

        map
    })
}

#[derive(Debug, Error)]
#[error("unknown error while encoding VDF")]
pub struct EncodeError;

pub fn encode(map: &Map) -> Result<Vec<u8>, DecodeError> {
    Ok({
        let mut bytes = Vec::new();

        for (key, value) in map {
            bytes.push(match value {
                Val::Map(_) => TYPE_MAP,
                Val::Str(_) => TYPE_STR,
                Val::Int(_) => TYPE_INT,
                Val::End => TYPE_END,
            });

            bytes.extend_from_slice(key.as_bytes_with_nul());

            match value {
                Val::Map(map) => bytes.extend_from_slice(&encode(map)?),
                Val::Str(str) => bytes.extend_from_slice(str.as_bytes_with_nul()),
                Val::Int(int) => bytes.extend_from_slice(&int.to_le_bytes()),
                Val::End => {}
            }
        }

        bytes.push(TYPE_END);

        bytes
    })
}
