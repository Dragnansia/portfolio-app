mod app;
mod image;
mod project;

use app::Portfolio;
use eframe::{run_native, NativeOptions};

fn main() {
    run_native(Box::new(Portfolio::default()), NativeOptions::default());
}
