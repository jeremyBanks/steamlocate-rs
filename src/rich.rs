use std::ffi::CString;


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

#[derive(Debug, Default, Clone, Copy)]
pub enum ShortcutLogoPosition {
    #[default]
    BottomLeft,
    TopCenter,
    CenterCenter,
    BottomCenter,
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
