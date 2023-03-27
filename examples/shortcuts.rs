//! Just prints all discovered shortcuts aka all non-Steam added games

use hypothetical_steam_shortcut_crate::SteamDir;

fn main() {
    let mut steamdir = SteamDir::locate().unwrap();
    let shortcuts = steamdir.shortcuts();
    println!("Shortcuts - {:#?}", shortcuts);
}
