#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    sovereign_notify_queue::run();
}
