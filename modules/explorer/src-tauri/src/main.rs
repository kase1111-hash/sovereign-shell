//! Sovereign Explorer — Main Entry Point.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    sovereign_explorer_lib::run();
}
