// sem console extra no release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    painel_lib::run()
}
