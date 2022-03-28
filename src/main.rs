mod app;
mod image;
mod link;
mod project;
mod state;

use app::Portfolio;
use eframe::{run_native, NativeOptions};

#[tokio::main]
async fn main() {
    run_native(Box::new(Portfolio::new().await), NativeOptions::default());
}
