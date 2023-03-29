//! `slinky` is a library for adding a shortcut to your binary to the local
//! Steam game/app library (without actually distributing it through Steam).
//!
//! At least for this initial version, the only supported platform is
//! Steam Deck Arch Linux, and the only entry point is the `slinky::linky!`
//! macro.
//!
//! The `slinky::linky!` macro is typically called near the beginning of
//! your `main` function. It takes optional keyword arguments. It returns
//! some `impl std::process::Termination`.
//!
//! The `slinky::start!` macro is similar, but instead of creating a shortcut
//! it's used to launch an existing shortcut or Steam game. It also returns
//! some `impl std::process::Termination` when the game process exits.
//!
//! ### Arguments
//!
//!
//!  
//! - `binary`
//!
//! ### Example
//!
//! ```
//! pub fn main() {
//!     slinky::linky! {
//!         name: "Celeste with Sync"
//!     };
//!
//!     slinky::start! {
//!         app_id: 504230,
//!     };
//! }
//! ```

#[doc(hidden)]
pub use crate as slinky;

use std::borrow::Cow;
use std::ops::Deref;
use std::ops::DerefMut;
use std::path::PathBuf;

/// The arguments to the `slinky::linky!` macro. All fields are optional.
#[derive(Default)]
pub struct Args {
    /// The steam app ID used for this shortcut.
    /// This can be any value with the high bit set (to indicate that it's a shortcut),
    /// but most tools prefer to use the same value that Steam would if it created the shortcut.
    ///
    /// ### Default
    ///  
    /// The value in the file `steam_appid.txt` in current crate's root directory, if any.
    ///
    /// Otherwise, calculated from `binary` and `name` using the same algorithm
    /// as the Steam client uses when adding shortcuts.
    pub app_id: Option<u32>,

    /// The desired application binary path. This is where the shortcut will point.
    ///
    /// ### Default
    ///
    /// `$HOME/.local/bin/$CARGO_CRATE_NAME`
    pub binary: Option<PathBuf>,

    /// The application name that will be displayed in the Steam UI.
    ///
    /// ### Default
    ///
    /// The file name component of the `binary` path.
    pub name: Option<String>,

    /// The existing/source application binary path. If no executable exists at the
    /// `binary` path, or the file contents differ, `binary_source` will be
    /// copied to `binary` before the shortcut is created or launched.
    ///
    /// ### Default
    ///
    /// The path to the current process' binary.
    pub binary_source: Option<PathBuf>,

    /// Whether this application must only run from the `binary` path.
    /// If `true` and the application is being run from another path, the
    /// process will be re-started running from the `binary` path.
    ///
    /// The new binary will replace the current process in-place.
    ///
    /// ### Default
    ///
    /// `true`, but note that it's effectively a no-op unless `binary` or
    /// `binary_source` are changed.
    pub must_run_from_binary_path: Option<bool>,

    /// Whether this application must only be run through Steam.
    /// If `true` and the application has been launched outside of Steam,
    /// the process will be re-launched through Steam. This supersedes
    /// `must_run_from_binary_path`.
    ///
    /// This is kind-of like calling the official Steamworks API function
    /// [`SteamAPI_RestartAppIfNecessary`](https://partner.steamgames.com/doc/api/steam_api#SteamAPI_RestartAppIfNecessary).
    ///
    /// The new binary will run in a new process. The current process will block
    /// until it the new process exits.
    ///
    /// ### Default
    ///
    /// `false`
    pub must_run_from_steam: Option<bool>,

    /// The arguments to use when re-launching the application.
    ///
    /// ### Default
    ///
    /// The current process's arguments.
    pub args: Option<Vec<String>>,

    /// The square icon to use for this shortcut in the Steam library. Can include transparency.
    pub png_square: Option<Cow<'static, [u8]>>,

    /// The portrait-aligned cover to use for this shortcut in the Steam library. Must be opaque.
    pub png_portrait: Option<Cow<'static, [u8]>>,

    /// The landscape-aligned cover to use for this shortcut in the Steam library. Must be opaque.
    pub png_landscape: Option<Cow<'static, [u8]>>,

    /// The hero image to use for this shortcut in the Steam library. Must be opaque.
    pub png_hero: Option<Cow<'static, [u8]>>,

    /// The logo image to use for this shortcut in the Steam library. Can include transparency.
    pub png_logo: Option<Cow<'static, [u8]>>,

    /// The position and maximum dimensions of the logo image over the hero image in the Steam library.
    ///
    /// ### Default
    ///
    /// [`ShortcutLogoPosition::BottomLeft`] with `50.0`% max-width and `50.0`% max-height.
    pub png_logo_placement: Option<(ShortcutLogoPosition, (f32, f32))>,

    /// The name of the crate this macro was invoked from.
    pub crate_name: &'static str,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum ShortcutLogoPosition {
    #[default]
    BottomLeft,
    TopCenter,
    CenterCenter,
    BottomCenter,
}

impl Args {
    pub fn linky(&self) {
        eprintln!("how do I linky?");
    }
}

#[doc(hidden)]
pub struct Linky(pub Args);

impl Deref for Linky {
    type Target = Args;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Linky {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for Linky {
    fn drop(&mut self) {
        self.0.linky()
    }
}

#[macro_export]
macro_rules! linky {
    {$(,)?} => {{
        $crate::Linky($crate::Args {
            crate_name: env!("CARGO_CRATE_NAME"),
            ..Default::default()
        })
    }};

    {
        name: $name:expr
        $(, $($rest:tt)*)?
    } => {{
        let mut linky = $crate::linky!{$($($rest)*)?};
        linky.name = Some($crate::cast![to owned String = $name]);
        linky
    }};

    {
        app_id: $app_id:expr
        $(, $($rest:tt)*)?
    } => {{
        let mut linky = $crate::linky!{$($($rest)*)?};
        linky.app_id = Some($crate::cast![u32 = $app_id]);
        linky
    }};

    {
        must_run_from_binary_path: $must_run_from_binary_path:expr
        $(, $($rest:tt)*)?
    } => {{
        let mut linky = $crate::linky!{$($($rest)*)?};
        linky.must_run_from_binary_path = Some($must_run_from_binary_path);
        linky
    }};

    {
        must_run_from_steam: $must_run_from_steam:expr
        $(, $($rest:tt)*)?
    } => {{
        let mut linky = $crate::linky!{$($($rest)*)?};
        linky.must_run_from_steam = Some($must_run_from_steam);
        linky
    }};

    {
        app_id from $path:literal
        $(, $($rest:tt)*)?
    } => {{
        let mut linky = $crate::linky!{$($($rest)*)?};
        linky.app_id = Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path)).trim().parse().expect("expected a valid u32 app ID"));
        linky
    }};

    {
        assets from $path:literal
        $(, $($rest:tt)*)?
    } => {{
        let mut linky = $crate::linky!{$($($rest)*)?};
        linky.png_square = Some(::std::borrow::Cow::Borrowed(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path, "_icon.png"))));
        linky.png_portrait = Some(::std::borrow::Cow::Borrowed(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path, "p.png"))));
        linky.png_landscape = Some(::std::borrow::Cow::Borrowed(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path, ".png"))));
        linky.png_hero = Some(::std::borrow::Cow::Borrowed(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path, "_hero.png"))));
        linky.png_logo = Some(::std::borrow::Cow::Borrowed(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path, "_logo.png"))));
        linky
    }};
}

/// Ascribes a type to an potentially ambiguously-typed expression.
#[macro_export]
macro_rules! cast {
    ($ty:ty = $expr:expr) => {{
        fn cast(value: $ty) -> $ty {
            value
        }
        cast($expr)
    }};

    (into $ty:ty = $expr:expr) => {{
        fn cast_into<Value: Into<$ty>>(value: Value) -> $ty {
            value.into()
        }
        cast_into($expr)
    }};

    (as ref to $ty:ty = $expr:expr) => {{
        fn cast_as_ref<Value: ?Sized + AsRef<$ty>>(value: &Value) -> &$ty {
            value.as_ref()
        }
        cast_as_ref($expr)
    }};

    (to owned $ty:ty = $expr:expr) => {{
        fn cast_to_owned<Value: ?Sized + ToOwned<Owned = $ty>>(value: &Value) -> $ty {
            value.to_owned()
        }
        cast_to_owned($expr)
    }};
}

// #[derive(Debug, Default)]
// pub struct Linky {
//     name: Option<String>,
// }

// impl Linky {
//     pub fn exec(self) {
//         drop(self)
//     }
// }

// impl Drop for Linky {
//     fn drop(&mut self) {
//         todo!()
//     }
// }
/*

pub mod library {
    //! Manipulating the Steam library shortcuts directly.

    #[derive(Debug, Clone)]
    pub struct Shortcut {
        pub app_id: u32,
        pub name: String,
        pub binary: PathBuf,
        pub working_directory: PathBuf,
    }

    #[derive(Debug, Clone, Default)]
    pub struct ShortcutAssets {
        pub icon: Option<Vec<u8>>,
        pub capsule: Option<Vec<u8>>,
        pub poster: Option<Vec<u8>>,
        pub hero: Option<Vec<u8>>,
        pub logo: Option<Vec<u8>>,
        pub logo_position: Option<ShortcutLogoPosition>,
        pub logo_max_height_percent: Option<f32>,
        pub logo_max_width_percent: Option<f32>,
    }

    impl Shortcut {
        pub fn new(binary: PathBuf, name: String) -> Self {
            let name = binary
                .file_name()
                .expect("binary path must have a file name")
                .to_string_lossy()
                .to_string();
            Shortcut::new_with_name(binary, name)
        }

        pub fn new_with_name_and_id(binary: PathBuf, name: String, app_id: u32) -> Self {
            Shortcut {
                app_id,
                name,
                binary,
                working_directory: None,
                icon: None,
                capsule: None,

            }
        }

        pub fn new_with_id(binary: PathBuf, app_id: u32) -> Self {
            Shortcut::new()
        }

        pub fn new_with_id(binary: PathBuf, app_id: u32) -> Self {
            Shortcut::new()
        }
    }

    pub fn default_app_id_for_name_and_binary(name: &str, binary: &Path) -> u32 {
        todo!()
    }

    pub fn upsert(shortcut: Shortcut) -> Result<(), ()> { todo!() }

    pub fn remove(app_id: u32) -> Result<(), ()> { todo!() }
}

use std::ffi::CString;

mod steam_config {
    macro_rules! App {
        {

        } => {

        };
    }
}


steam_config::app! {

}

// why are you adding configuration instead of just writing fucking code

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RunThroughSteam {
    Require,
    #[default]
    Attempt,
    Allow,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InstallLocation {
    /// Leave the binary where it is.
    None,
    /// Install the binary
    UserLocal,
}

#[derive(Debug, Default, Clone)]
#[allow(non_snake_case)]
pub struct ShortcutBuilder<'a> {
    id: Option<u32>,
    name: Option<CString>,
    exe: Option<CString>,
    icon: Option<&'a [u8]>,
    capsule: Option<&'a [u8]>,
    poster: Option<&'a [u8]>,
    hero: Option<&'a [u8]>,
    logo: Option<&'a [u8]>,
    logo_position: Option<ShortcutLogoPosition>,
    logo_max_height_percent: Option<f32>,
    logo_max_width_percent: Option<f32>,
}

// {"nVersion":1,"logoPosition":{"pinnedPosition":"UpperCenter","nWidthPct":95.70661896243291,"nHeightPct":82.63888888888891}}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    steam_shortcuts::create()
        .with_name("Celeste 🍓")
        .with_exe("/usr/bin/celeste")
        .with_icon(b"beep boop im a png")
        .save();

    steam_shortcuts::find()
        .with_name("Celeste 🍓")
        .update()
        .with_name("Celeste Plus")
}
*/
