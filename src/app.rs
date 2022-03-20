use crate::image::new_image_file_dialog;
use eframe::{
    egui::{self, Context, Layout, Slider, Window},
    epi,
};
use std::{
    sync::{
        mpsc::{channel, Receiver},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

pub enum Response {
    Image(egui::TextureHandle),
    Nothing,
}

pub struct Portfolio {
    images: Vec<egui::TextureHandle>,
    max_image_width: f32,

    tasks: Vec<(Receiver<Response>, JoinHandle<()>)>,
}

impl Default for Portfolio {
    fn default() -> Self {
        Self {
            max_image_width: 160f32,
            images: vec![],
            tasks: vec![],
        }
    }
}

impl epi::App for Portfolio {
    fn update(&mut self, ctx: &egui::Context, _: &epi::Frame) {
        self.check_tasks();
        self.images(ctx);
        self.debug(ctx);
    }

    fn name(&self) -> &str {
        "Portfolio App"
    }
}

impl Portfolio {
    fn check_tasks(&mut self) {
        self.tasks.retain(|t| {
            if let Ok(response) = t.0.try_recv() {
                match response {
                    Response::Image(image) => {
                        self.images.push(image);
                    }
                    Response::Nothing => {}
                };

                false
            } else {
                true
            }
        });
    }

    fn debug(&mut self, ctx: &Context) {
        Window::new("Portfolio Data").show(ctx, |ui| {
            ui.label(&format!("Worker: {}", self.tasks.len()));

            ui.add(Slider::new(&mut self.max_image_width, 0f32..=1000f32));
        });
    }

    fn images(&mut self, ctx: &Context) {
        Window::new("Images").show(ctx, |ui| {
            let layout = Layout::top_down(eframe::emath::Align::Center);
            ui.with_layout(layout, |ui| {
                if ui.button("Add Image").clicked() {
                    let context = Arc::new(Mutex::new(ui.ctx().clone()));
                    let (sender, receiver) = channel();

                    self.tasks.push((
                        receiver,
                        thread::spawn(move || {
                            // Do this on other thread
                            new_image_file_dialog(sender, context);
                        }),
                    ));
                }

                for image in &self.images {
                    let img_size = self.max_image_width * image.size_vec2() / image.size_vec2().y;
                    ui.image(image, img_size);
                }
            });
        });
    }
}
