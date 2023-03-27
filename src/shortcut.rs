//! **WARN:** This is all hacky and should be replaced with proper binary VDF parsing

use std::{fs, iter::Peekable, path::Path, slice::Iter};

/// A added non-Steam game
///
/// Information is parsed from your `userdata/<user_id>/config/shortcuts.vdf` files
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct Shortcut {
    /// Steam's short-format (32-bit) app ID for this shortcut.
    ///
    /// This is the format used for naming associated image files in the
    /// `Steam/userdata/USER_ID/config/grid directory`.
    pub appid: u32,
    /// The name of the application
    pub app_name: String,
    /// The executable used to launch the app
    ///
    /// This is either the name of the program or the full path to the program
    pub executable: String,
    /// The directory that the application should be run in
    pub start_dir: String,
}

impl Shortcut {
    /// Creates a new Shortcut with the given name and executable path,
    /// generating the same app ID that Steam would.
    pub fn new(app_name: String, executable: String) -> Shortcut {
        let algorithm = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);

        // This is the same algorithm that Steam uses to generate the default
        // app ID for shortcuts added through the UI. This ID does not change,
        // even if users change the name or executable path later.
        let mut digest = algorithm.digest();
        digest.update(executable.as_bytes());
        digest.update(app_name.as_bytes());
        let appid = digest.finalize() | 0x80000000;

        let executable_path = Path::new(&executable);
        let start_dir = executable_path
            .parent()
            .unwrap_or(&executable_path)
            .to_str()
            .unwrap()
            .to_string();

        Shortcut {
            appid,
            app_name,
            executable,
            start_dir,
        }
    }

    /// Calculates the shortcut's long-format (64-bit) ID for Steam.
    ///
    /// This is the format used in `steam://rungameid/...` URLs.
    pub fn steam_id(&self) -> u64 {
        ((self.appid as u64) << 32) | 0x02000000
    }

    /// Saves this shortcut to the Steam library of the given user ID, or all Steam libraries if `None`.
    ///
    /// This will either insert or update depending on whether a shortcut with the same app ID already exists.
    ///
    /// ```
    /// let shortcut = Shortcut::new("My Game".to_string(), "C:\\Program Files\\My Game\\MyGame.exe".to_string());
    ///
    /// shortcut.save_to_library(None)
    /// ```
    pub fn save_to_library(&self, user_id: Option<u64>) {
        let steam_dir = crate::SteamDir::locate().unwrap();

        let user_data = steam_dir.path.join("userdata");
        for entry in fs::read_dir(user_data).ok().unwrap().filter_map(|e| e.ok()) {
            if let Some(user_id) = user_id {
                if entry.file_name().to_string_lossy() != user_id.to_string() {
                    continue;
                }
            }

            let shortcuts_path = entry.path().join("config").join("shortcuts.vdf");
            if !shortcuts_path.is_file() {
                continue;
            }

            println!("let's do it!");
        }
    }
}

#[cfg(not(feature = "steamid_ng"))]
type SteamID = u64;
#[cfg(feature = "steamid_ng")]
type SteamID = steamid_ng::SteamID;

/// Discovers any shorcuts stored within `userdata`
pub fn discover_shortcuts(steam_dir: &Path) -> Vec<Shortcut> {
    fn inner(steam_dir: &Path) -> Option<Vec<Shortcut>> {
        let mut shortcuts = Vec::new();

        // Find and parse each `userdata/<user_id>/config/shortcuts.vdf` file
        let user_data = steam_dir.join("userdata");
        for entry in fs::read_dir(user_data).ok()?.filter_map(|e| e.ok()) {
            let shortcuts_path = entry.path().join("config").join("shortcuts.vdf");
            if !shortcuts_path.is_file() {
                continue;
            }

            if let Ok(contents) = fs::read(&shortcuts_path) {
                if let Some(parsed) = parse_shortcuts(&contents) {
                    shortcuts.extend(parsed);
                }
            }
        }

        Some(shortcuts)
    }

    inner(steam_dir).unwrap_or_default()
}

/// Advances `it` until right after the matching `needle`
///
/// Only works if the starting byte is not used anywhere else in the needle. This works well when
/// finding keys since the starting byte indicates the type and wouldn't be used in the key
#[must_use]
fn after_many_case_insensitive(it: &mut Peekable<Iter<u8>>, needle: &[u8]) -> bool {
    loop {
        loop {
            let mut needle_it = needle.iter();
            let b = match it.next() {
                Some(b) => b,
                None => return false,
            };

            let maybe_needle_b = needle_it.next();
            if maybe_u8_eq_ignore_ascii_case(maybe_needle_b, Some(b)) {
                loop {
                    if needle_it.len() == 0 {
                        return true;
                    }

                    let maybe_b = it.peek();
                    let maybe_needle_b = needle_it.next();
                    if maybe_u8_eq_ignore_ascii_case(maybe_needle_b, maybe_b.copied()) {
                        let _ = it.next();
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

fn maybe_u8_eq_ignore_ascii_case(maybe_b1: Option<&u8>, maybe_b2: Option<&u8>) -> bool {
    maybe_b1
        .zip(maybe_b2)
        .map(|(b1, b2)| b1.eq_ignore_ascii_case(b2))
        .unwrap_or_default()
}

fn parse_value_str(it: &mut Peekable<Iter<u8>>) -> Option<String> {
    let mut buff = Vec::new();
    loop {
        let b = it.next()?;
        if *b == 0x00 {
            break Some(String::from_utf8_lossy(&buff).into_owned());
        }

        buff.push(*b);
    }
}

fn parse_value_u32(it: &mut Peekable<Iter<u8>>) -> Option<u32> {
    let bytes = [*it.next()?, *it.next()?, *it.next()?, *it.next()?];
    Some(u32::from_le_bytes(bytes))
}

// The performance of this is likely terrible, but also the files we're parsing are tiny so it
// won't matter
fn parse_shortcuts(contents: &[u8]) -> Option<Vec<Shortcut>> {
    let mut it = contents.iter().peekable();
    let mut shortcuts = Vec::new();

    loop {
        if !after_many_case_insensitive(&mut it, b"\x02appid\x00") {
            return Some(shortcuts);
        }
        let appid = parse_value_u32(&mut it)?;

        if !after_many_case_insensitive(&mut it, b"\x01AppName\x00") {
            return None;
        }
        let app_name = parse_value_str(&mut it)?;

        if !after_many_case_insensitive(&mut it, b"\x01Exe\x00") {
            return None;
        }
        let executable = parse_value_str(&mut it)?;

        if !after_many_case_insensitive(&mut it, b"\x01StartDir\x00") {
            return None;
        }
        let start_dir = parse_value_str(&mut it)?;

        let shortcut = Shortcut {
            appid,
            app_name,
            executable,
            start_dir,
        };
        shortcuts.push(shortcut);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        let contents = include_bytes!("../tests/sample_data/shortcuts.vdf");
        let shortcuts = parse_shortcuts(contents).unwrap();
        assert_eq!(
            shortcuts,
            vec![
                Shortcut {
                    appid: 2786274309,
                    app_name: "Anki".into(),
                    executable: "\"anki\"".into(),
                    start_dir: "\"./\"".into(),
                },
                Shortcut {
                    appid: 2492174738,
                    app_name: "LibreOffice Calc".into(),
                    executable: "\"libreoffice\"".into(),
                    start_dir: "\"./\"".into(),
                },
                Shortcut {
                    appid: 3703025501,
                    app_name: "foo.sh".into(),
                    executable: "\"/usr/local/bin/foo.sh\"".into(),
                    start_dir: "\"/usr/local/bin/\"".into(),
                }
            ],
        );

        let contents = include_bytes!("../tests/sample_data/shortcuts_different_key_case.vdf");
        let shortcuts = parse_shortcuts(contents).unwrap();
        assert_eq!(
            shortcuts,
            vec![Shortcut {
                appid: 2931025216,
                app_name: "Second Life".into(),
                executable: "\"/Applications/Second Life Viewer.app\"".into(),
                start_dir: "\"/Applications/\"".into(),
            }]
        );
    }

    #[cfg(feature = "shortcuts_extras")]
    #[test]
    fn shortcuts_extras() {
        let contents = include_bytes!("../tests/sample_data/shortcuts.vdf");
        let shortcuts = parse_shortcuts(contents).unwrap();
        let ideal_ids = vec![0xe89614fe02000000, 0xdb01c79902000000, 0x9d55017302000000];
        for (id, shortcut) in ideal_ids.into_iter().zip(shortcuts.iter()) {
            assert_eq!(id, shortcut.steam_id());
        }
    }
}
