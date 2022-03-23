use crate::{
    image::{new_image_file_dialog, Image},
    project::Project,
    state::State,
};
use eframe::{
    egui::{self, Context, Layout, Slider, Spinner, Window},
    epi,
};
use futures::TryStreamExt;
use mongodb::{options::ClientOptions, Client, Database};
use std::sync::{
    mpsc::{channel, Receiver},
    Arc, Mutex,
};
use tokio::task::JoinHandle;

#[derive(Debug, PartialEq)]
pub enum Response {
    Nothing,
}

pub struct Portfolio {
    images: Vec<Arc<Mutex<Image>>>,
    max_image_width: f32,
    projects: Vec<Project>,

    tasks: Vec<(Receiver<Response>, JoinHandle<()>)>,

    db: Database,
}

impl epi::App for Portfolio {
    fn update(&mut self, ctx: &egui::Context, _: &epi::Frame) {
        self.check_tasks();
        self.images(ctx);
        self.debug(ctx);
        self.projects(ctx);
    }

    fn name(&self) -> &str {
        "Portfolio App"
    }
}

impl Portfolio {
    pub async fn new() -> Self {
        let client = Self::connect_to_database().await.unwrap();

        let db = client.database("portfolio");
        let projects = Self::projects_list(&db).await;

        Self {
            max_image_width: 160f32,
            images: vec![],
            tasks: vec![],
            projects,
            db,
        }
    }

    async fn connect_to_database() -> Result<Client, mongodb::error::Error> {
        let url = "mongodb://localhost:27017";
        let options = ClientOptions::parse(url).await?;

        Client::with_options(options)
    }

    async fn projects_list(db: &Database) -> Vec<Project> {
        db.collection::<Project>("projects")
            .find(None, None)
            .await
            .unwrap()
            .try_collect::<Vec<_>>()
            .await
            .unwrap()
    }

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
                    let image = Arc::new(Mutex::new(Image::new()));
                    self.images.push(image.clone());

                    self.tasks.push((
                        receiver,
                        tokio::spawn(async move {
                            new_image_file_dialog(sender, image, context).await;
                        }),
                    ));
                }

                for (index, image) in self.images.clone().iter().enumerate() {
                    if let Ok(mut image) = image.lock() {
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
                            ui.label(&image.name);
                            ui.text_edit_singleline(&mut image.alt);
                        }
                    }
                }
            });
        });
    }

    fn projects(&mut self, ctx: &Context) {
        Window::new("projects").show(ctx, |ui| {
            for project in &mut self.projects {
                ui.label(&project.name);
                ui.text_edit_singleline(&mut project.description);
                ui.add_space(30f32);
            }
        });
    }
}
