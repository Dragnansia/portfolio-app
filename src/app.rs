use crate::{
    image::{new_image_file_dialog, Image},
    link::{Link, LinkIcon},
    project::Project,
};
use const_env::from_env;
use eframe::{
    egui::{self, CollapsingHeader, Context, Layout, Slider, Window},
    epi,
};
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::ClientOptions,
    Client, Database,
};
use std::sync::{
    mpsc::{channel, Receiver},
    Arc, Mutex,
};
use tokio::task::JoinHandle;

type Tasks = Vec<(Receiver<Response>, JoinHandle<()>)>;

#[from_env]
const MONGO_URL: &str = "mongodb://localhost:27017";

#[derive(Debug, PartialEq)]
pub enum Response {
    Nothing,
    Image(Image),
    Project(Project),
    RemoveProject(ObjectId),
    Error(String),
}

pub struct Portfolio {
    max_image_width: f32,
    projects: Vec<Project>,
    project_id: Option<usize>,
    tasks: Tasks,
    db: Database,

    errors: Vec<String>,
}

impl epi::App for Portfolio {
    fn update(&mut self, ctx: &egui::Context, _: &epi::Frame) {
        self.check_tasks();
        self.debug_window(ctx);
        self.projects_window(ctx);
        self.project_window(ctx);
    }

    fn name(&self) -> &str {
        "Portfolio App"
    }
}

impl Portfolio {
    pub async fn new() -> Portfolio {
        let client = Self::connect_to_database().await.unwrap();

        let db = client.database("portfolio");
        let projects = Self::projects_list(&db).await;

        Self {
            max_image_width: 160f32,
            tasks: vec![],
            errors: vec![],
            project_id: None,
            projects,
            db,
        }
    }

    async fn connect_to_database() -> Result<Client, mongodb::error::Error> {
        let options = ClientOptions::parse(MONGO_URL).await?;

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
                    Response::Image(img) => {
                        if let Some(index) = self.project_id {
                            let project = &mut self.projects[index];
                            project.images.push(img);
                        }
                    }
                    Response::Error(err) => {
                        self.errors.push(err);
                    }
                    Response::Project(project) => {
                        self.projects.push(project);
                    }
                    Response::RemoveProject(id) => {
                        let pos = self.projects.iter().position(|p| p.id == id);
                        if let Some(pos) = pos {
                            self.projects.remove(pos);

                            if let Some(project_id) = self.project_id {
                                if pos == project_id {
                                    self.project_id = None;
                                }
                            }
                        }
                    }
                };

                false
            } else {
                true
            }
        });
    }

    fn debug_window(&mut self, ctx: &Context) {
        Window::new("Portfolio Data").show(ctx, |ui| {
            CollapsingHeader::new("Datas").show(ui, |ui| {
                ui.label(&format!("Worker: {}", self.tasks.len()));
                ui.add(Slider::new(&mut self.max_image_width, 0f32..=1000f32));
            });

            CollapsingHeader::new("Errors").show(ui, |ui| {
                for err in &self.errors {
                    ui.label(err);
                }
            });
        });
    }

    fn images_window(task: &mut Tasks, width: f32, project: &mut Project, ctx: &Context) {
        let layout = Layout::top_down(eframe::emath::Align::Center);
        Window::new("Project Image(s)").show(ctx, |ui| {
            ui.with_layout(layout, |ui| {
                if ui.button("Add Image").clicked() {
                    let context = Arc::new(Mutex::new(ui.ctx().clone()));
                    let (sender, receiver) = channel();

                    task.push((
                        receiver,
                        tokio::spawn(async move {
                            new_image_file_dialog(sender, context).await;
                        }),
                    ));
                }

                let images = &mut project.images;
                images.retain_mut(|image| {
                    let data = image.egui_data.as_ref().unwrap();

                    let img_size = width * data.size_vec2() / data.size_vec2().y;
                    let mut is_click = false;
                    ui.image(data, img_size).context_menu(|ui| {
                        if ui.button("Remove").clicked() {
                            is_click = true;
                            ui.close_menu();
                        }
                    });
                    ui.label(&image.name);
                    ui.text_edit_singleline(&mut image.alt);

                    !is_click
                });
            });
        });
    }

    fn links_window(project: &mut Project, ctx: &Context) {
        Window::new("Project Link(s)").show(ctx, |ui| {
            let layout = Layout::top_down(eframe::emath::Align::Center);
            ui.with_layout(layout, |ui| {
                ui.menu_button("Add new Link", |ui| {
                    if ui.button("Image").clicked() {
                        project.links.push(Link {
                            url: String::new(),
                            icon: LinkIcon::Image(String::new()),
                        });

                        ui.close_menu();
                    }

                    if ui.button("FontAwesome").clicked() {
                        project.links.push(Link {
                            url: String::new(),
                            icon: LinkIcon::FontAwesome(String::new()),
                        });

                        ui.close_menu();
                    }
                });
            });

            ui.add_space(10f32);

            let links = &mut project.links;
            links.retain_mut(|link| {
                ui.horizontal(|ui| {
                    ui.label("Url:");
                    ui.text_edit_singleline(&mut link.url);
                });

                match &mut link.icon {
                    LinkIcon::Image(link) => {
                        ui.horizontal(|ui| {
                            ui.label("Image link:");
                            ui.text_edit_singleline(link);
                        });
                    }
                    LinkIcon::FontAwesome(icon) => {
                        ui.horizontal(|ui| {
                            ui.label("FontAwesome icon:");
                            ui.text_edit_singleline(icon);
                        });
                    }
                }

                let mut is_click = false;
                if ui.button("Remove").clicked() {
                    is_click = true;
                }

                ui.add_space(5f32);

                !is_click
            });
        });
    }

    fn projects_window(&mut self, ctx: &Context) {
        Window::new("projects").show(ctx, |ui| {
            let layout = Layout::top_down_justified(eframe::emath::Align::Center);
            ui.with_layout(layout, |ui| {
                if ui.button("Add new project").clicked() {
                    let (sender, receiver) = channel();
                    let collection = self.db.collection::<Project>("projects");

                    self.tasks.push((
                        receiver,
                        tokio::spawn(async move {
                            let project = Project::new();
                            collection.insert_one(project.clone(), None).await.unwrap();

                            sender.send(Response::Project(project)).unwrap();
                        }),
                    ));
                }

                ui.add_space(15f32);

                for (index, project) in &mut self.projects.iter().enumerate() {
                    let but = ui.button(&project.name);
                    if but.clicked() {
                        self.project_id = Some(index);
                    }

                    but.context_menu(|ui| {
                        if ui.button("Remove").clicked() {
                            let (sender, receiver) = channel();
                            let collection = self.db.collection::<Project>("projects");
                            let project = project.clone();

                            self.tasks.push((
                                receiver,
                                tokio::spawn(async move {
                                    let res = collection
                                        .find_one_and_delete(
                                            doc! {
                                                "_id": project.id
                                            },
                                            None,
                                        )
                                        .await;

                                    if let Err(error) = res {
                                        sender.send(Response::Error(error.to_string())).unwrap();
                                    } else {
                                        sender.send(Response::RemoveProject(project.id)).unwrap();
                                    }
                                }),
                            ));

                            ui.close_menu();
                        }
                    });
                }
            });
        });
    }

    fn project_window(&mut self, ctx: &Context) {
        if let Some(index) = self.project_id {
            let project = self.projects.get_mut(index).unwrap();

            Self::images_window(&mut self.tasks, self.max_image_width, project, ctx);
            Self::links_window(project, ctx);

            Window::new("Select Project").show(ctx, |ui| {
                ui.label(&format!("{:?}", project.id));
                ui.add_space(5f32);

                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut project.name)
                });
                ui.add_space(3f32);

                ui.label("Description:");
                ui.text_edit_multiline(&mut project.description);

                ui.add_space(15f32);
                if ui.button("Update").clicked() {
                    let collection = self.db.collection::<Project>("projects");
                    let (sender, receiver) = channel();
                    let project = project.clone();

                    self.tasks.push((
                        receiver,
                        tokio::spawn(async move {
                            let res = collection
                                .find_one_and_update(
                                    doc! { "_id": project.id },
                                    doc! { "$set": project.doc() },
                                    None,
                                )
                                .await;

                            if let Some(err) = res.err() {
                                sender
                                    .send(Response::Error(format!("{:#?}", err.kind)))
                                    .unwrap();
                            } else {
                                sender.send(Response::Nothing).unwrap();
                            }
                        }),
                    ));
                }
            });
        }
    }
}
