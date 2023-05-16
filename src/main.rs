#![cfg(feature = "gui-native")]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    vrc_canvas::run_gui_native().await
}
