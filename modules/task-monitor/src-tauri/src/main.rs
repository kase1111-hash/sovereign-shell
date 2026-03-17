#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    sovereign_task_monitor_lib::run();
}
