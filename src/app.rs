use crate::{
    image::{new_image_file_dialog, Image},
    state::State,
};
use eframe::{
    egui::{self, Context, Layout, Slider, Spinner, Window},
    epi,
};
use std::sync::{
    mpsc::{channel, Receiver},
    Arc, Mutex,
};
use tokio::task::JoinHandle;

pub enum Response {
    Nothing,
}

pub struct Portfolio {
    images: Vec<Arc<Mutex<Image>>>,
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
                    let image = Arc::new(Mutex::new(Image::default()));
                    self.images.push(image.clone());

                    self.tasks.push((
                        receiver,
                        tokio::spawn(async move {
                            new_image_file_dialog(sender, image, context).await;
                        }),
                    ));
                }

                for (index, image) in self.images.clone().iter().enumerate() {
                    if let Ok(image) = image.lock() {
                        if image.state == State::Loading {
                            ui.add(Spinner::new().size(25f32));
                            ui.label(&image.name);
                        } else if let Some(data) = &image.data {
                            let img_size =
                                self.max_image_width * data.size_vec2() / data.size_vec2().y;
                            ui.image(data, img_size).context_menu(|ui| {
                                if ui.button("Remove").clicked() {
                                    let _ = self.images.remove(index);
                                }
                            });
                        }
                    }
                }
            });
        });
    }
}
