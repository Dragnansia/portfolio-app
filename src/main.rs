#![feature(vec_retain_mut)]

mod app;
mod image;
mod link;
mod project;

use app::Portfolio;
use eframe::{run_native, NativeOptions};

#[tokio::main]
async fn main() {
    run_native(Box::new(Portfolio::new().await), NativeOptions::default());
}
