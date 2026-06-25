use chrono::{DateTime, Local};
use eframe::egui::*;
use std::collections::BTreeMap;
use std::mem;
use uuid::Uuid;

#[derive(Copy, Clone)]
enum State {
    ModalNewProject,
    ModalNewTodo(ProjectId),
    ModalNewDesc(ProjectId, TodoId),

    SwitchProject(ProjectId, Option<TodoId>),
    SwitchTodo(ProjectId, TodoId),
    MarkTodoForDeletion(ProjectId, TodoId),
    MarkProjectForDeletion(ProjectId),

    Home(Option<TodoId>),
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
struct ProjectId(Uuid);

struct Project {
    title: String,
    todos: BTreeMap<TodoId, Todo>,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
struct TodoId(Uuid);

struct Todo {
    title: String,
    desc: Vec<Description>,
    done: bool,
    created: DateTime<Local>,
    // could have used done: Option<DateTime<Local>> but then we'll lose ui.checkbox(&mut done, ...) below
    completed: Option<DateTime<Local>>,
}

struct Description {
    description: String,
    created: DateTime<Local>,
}

pub struct App {
    projects: BTreeMap<ProjectId, Project>,

    // frame data
    state: Option<State>,
    active_proj: Option<ProjectId>,
    input_buffer: String,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "Noto".to_owned(),
            std::sync::Arc::new(FontData::from_static(include_bytes!(
                "../assets/NotoMono-Regular.ttf"
            ))),
        );

        // Put my font first (highest priority):
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "Noto".to_owned());

        // Put my font as last fallback for monospace:
        fonts
            .families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .push("Noto".to_owned());

        let mut style = (*cc.egui_ctx.global_style()).clone();
        style.text_styles = [
            (
                TextStyle::Heading,
                FontId::new(30.0, FontFamily::Proportional),
            ),
            (TextStyle::Body, FontId::new(18.0, FontFamily::Proportional)),
            (
                TextStyle::Monospace,
                FontId::new(14.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Button,
                FontId::new(14.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Small,
                FontId::new(10.0, FontFamily::Proportional),
            ),
        ]
        .into();
        cc.egui_ctx.set_global_style(style);

        Self {
            state: None,
            projects: BTreeMap::new(),
            active_proj: None,
            input_buffer: String::new(),
        }
    }

    fn top_menu(&mut self, ui: &mut Ui) {
        Panel::top("top_menu").show_inside(ui, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Projects", |ui| {
                    if ui.button("Create New Project").clicked() {
                        self.state = Some(State::ModalNewProject);
                    }
                    ui.separator();

                    for (id, projx) in self.projects.iter() {
                        if ui.button(&projx.title).clicked() {
                            self.state = Some(State::SwitchProject(*id, None));
                        }
                    }
                });
            });
        });
    }

    fn bottom_panel(&mut self, ui: &mut Ui) {
        Panel::bottom("bottom_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let label = match self.active_proj {
                    None => "Select Project",
                    Some(proj_id) => {
                        let proj = self.projects.get(&proj_id).unwrap();
                        &proj.title
                    }
                };
                ui.label(label);

                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    if let Some(active_proj) = self.active_proj
                        && ui.button("Delete Project").clicked()
                    {
                        self.state = Some(State::MarkProjectForDeletion(active_proj));
                    }
                });
            });
        });
    }

    fn todo_sidebar(&mut self, ui: &mut Ui) {
        Panel::left("todo_list")
            .size_range(Rangef::new(
                ui.available_width() / 6.,
                ui.available_width() / 3.,
            ))
            .show_inside(ui, |ui| {
                ui.take_available_width();
                Panel::bottom("completed_panel")
                    .resizable(true)
                    .show_inside(ui, |ui| {
                        ui.collapsing("completed todos", |ui| {
                            ui.take_available_height();
                            if let Some(proj_id) = self.active_proj {
                                let proj = self.projects.get_mut(&proj_id).unwrap();
                                ScrollArea::vertical().show(ui, |ui| {
                                    for (idx, todo) in proj.todos.iter_mut() {
                                        ui.horizontal_top(|ui| {
                                            if todo.done {
                                                let changed_response = ui.checkbox(&mut todo.done, "");
                                                if changed_response.changed() {
                                                    todo.completed = None;
                                                }

                                                if ui
                                                    .add(
                                                        Label::new(
                                                            RichText::new(&todo.title).weak(),
                                                        )
                                                        .sense(Sense::click())
                                                        .selectable(false),
                                                    )
                                                    .clicked()
                                                {
                                                    self.state =
                                                        Some(State::SwitchTodo(proj_id, *idx));
                                                }

                                                ui.with_layout(
                                                    Layout::right_to_left(Align::Min),
                                                    |ui| {
                                                        if ui
                                                            .add(
                                                                Label::new(
                                                                    RichText::new("X").heading(),
                                                                )
                                                                .sense(Sense::click())
                                                                .selectable(false),
                                                            )
                                                            .clicked()
                                                        {
                                                            self.state =
                                                                Some(State::MarkTodoForDeletion(
                                                                    proj_id, *idx,
                                                                ));
                                                        }
                                                    },
                                                );
                                            }
                                        });
                                    }
                                });
                            }
                        });
                    });

                ScrollArea::vertical().show(ui, |ui| {
                    if let Some(proj_id) = self.active_proj {
                        if ui.button("+").clicked() {
                            self.state = Some(State::ModalNewTodo(proj_id));
                        }
                        let proj = self.projects.get_mut(&proj_id).unwrap();
                        for (idx, todo) in proj.todos.iter_mut().rev() {
                            ui.horizontal_top(|ui| {
                                if !todo.done {
                                    let changed_response = ui.checkbox(&mut todo.done, "");
                                    if changed_response.changed() {
                                        todo.completed = Some(Local::now());
                                    }

                                    if ui
                                        .add(
                                            Label::new(RichText::new(&todo.title).heading())
                                                .sense(Sense::click())
                                                .selectable(false),
                                        )
                                        .clicked()
                                    {
                                        self.state = Some(State::SwitchTodo(proj_id, *idx));
                                    }

                                    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                        if ui
                                            .add(
                                                Label::new(RichText::new("X").heading())
                                                    .sense(Sense::click())
                                                    .selectable(false),
                                            )
                                            .clicked()
                                        {
                                            self.state =
                                                Some(State::MarkTodoForDeletion(proj_id, *idx));
                                        }
                                    });
                                }
                            });
                        }
                    } else {
                        ui.label("Select A Project");
                    }
                });
            });
    }

    fn new_project(&mut self, ui: &mut Ui) {
        let modal = Modal::new(Id::from("new project modal")).show(ui, |ui| {
            let response = ui.text_edit_singleline(&mut self.input_buffer);
            if self.input_buffer.trim().is_empty() {
                response.request_focus();
            }
            let success = response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
            if (success || ui.button("Create").clicked()) && !self.input_buffer.trim().is_empty() {
                let new_proj_id = ProjectId(Uuid::now_v7());
                let new_proj = Project {
                    title: mem::take(&mut self.input_buffer),
                    todos: BTreeMap::new(),
                };
                self.projects.insert(new_proj_id, new_proj);

                self.state = Some(State::SwitchProject(new_proj_id, None));
                self.active_proj = Some(new_proj_id);
                ui.close();
            }
        });

        if modal.should_close() && matches!(self.state, Some(State::ModalNewProject)) {
            self.state = Some(State::Home(None));
        }
    }

    fn new_todo(&mut self, ui: &mut Ui, proj_id: ProjectId) {
        let modal = Modal::new(Id::from("new todo modal")).show(ui, |ui| {
            let response = ui.text_edit_singleline(&mut self.input_buffer);
            if self.input_buffer.trim().is_empty() {
                response.request_focus();
            }
            let success = response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
            if (success || ui.button("Create").clicked()) && !self.input_buffer.trim().is_empty() {
                let new_todo_id = TodoId(Uuid::now_v7());
                let new_todo = Todo {
                    title: mem::take(&mut self.input_buffer),
                    desc: Vec::new(),
                    done: false,
                    created: Local::now(),
                    completed: None,
                };
                let proj = self.projects.get_mut(&proj_id).unwrap();
                proj.todos.insert(new_todo_id, new_todo);

                self.state = Some(State::SwitchTodo(proj_id, new_todo_id));
                ui.close();
            }
        });

        if modal.should_close() && matches!(self.state, Some(State::ModalNewTodo(_))) {
            self.state = Some(State::Home(None));
        }
    }

    fn new_desc(&mut self, ui: &mut Ui, proj_id: ProjectId, todo_id: TodoId) {
        let modal = Modal::new(Id::from("new description modal")).show(ui, |ui| {
            let response = ui.text_edit_multiline(&mut self.input_buffer);
            if self.input_buffer.trim().is_empty() {
                response.request_focus();
            }
            let success = response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
            if (success || ui.button("Create").clicked()) && !self.input_buffer.trim().is_empty() {
                let proj = self.projects.get_mut(&proj_id).unwrap();
                let todo = proj.todos.get_mut(&todo_id).unwrap();
                todo.desc.push(Description {
                    description: mem::take(&mut self.input_buffer),
                    created: Local::now(),
                });

                self.input_buffer.clear();
                self.state = Some(State::Home(Some(todo_id)));
                ui.close();
            }
        });

        if modal.should_close() && matches!(self.state, Some(State::ModalNewDesc(_, _))) {
            self.state = Some(State::Home(Some(todo_id)));
        }
    }

    fn confirm_todo_deletion(&mut self, ui: &mut Ui, proj_id: ProjectId, todo_id: TodoId) {
        let modal = Modal::new(Id::from("confirm todo delete")).show(ui, |ui| {
            ui.add(Label::new("Confirm delete?").selectable(false));
            let response = ui.horizontal(|ui| {
                if ui
                    .button(RichText::new("Delete").color(Color32::LIGHT_RED))
                    .clicked()
                {
                    let proj = self.projects.get_mut(&proj_id).unwrap();
                    proj.todos.remove(&todo_id);
                    self.state = None;
                    ui.close();
                }

                if ui.button("Cancel").clicked() {
                    self.state = None;
                    ui.close();
                }
            });
            response.response.request_focus();
        });

        if modal.should_close() && matches!(self.state, Some(State::MarkTodoForDeletion(_, _))) {
            self.state = Some(State::Home(Some(todo_id)));
        }
    }

    fn confirm_project_deletion(&mut self, ui: &mut Ui, proj_id: ProjectId) {
        let modal = Modal::new(Id::from("confirm project delete")).show(ui, |ui| {
            ui.add(Label::new("Confirm delete?").selectable(false));
            let response = ui.horizontal(|ui| {
                if ui
                    .button(RichText::new("Delete").color(Color32::LIGHT_RED))
                    .clicked()
                {
                    self.projects.remove(&proj_id);
                    self.state = None;
                    ui.close();
                }

                if ui.button("Cancel").clicked() {
                    self.state = Some(State::Home(None));
                    ui.close();
                }
            });
            response.response.request_focus();
        });

        if modal.should_close() && matches!(self.state, Some(State::MarkProjectForDeletion(_))) {
            self.state = Some(State::Home(None));
        }
    }

    fn descriptions_central(&mut self, ui: &mut Ui, proj_id: ProjectId, todo_id: TodoId) {
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            if ui.button("+").clicked() {
                self.state = Some(State::ModalNewDesc(proj_id, todo_id));
            }
        });
        let proj = self.projects.get(&proj_id).unwrap();
        let todo = proj.todos.get(&todo_id).unwrap();

        ui.label(format!("{}", todo_id.0));
        ui.separator();
        if todo.done {
            let delta = todo.completed.unwrap() - todo.created;
            ui.add(Label::new(format!("Completed in {} days", delta.num_days())).selectable(false));
        } else {
            let delta = Local::now() - todo.created;
            ui.add(Label::new(format!("Open for {} days", delta.num_days())).selectable(false));
        }
        for desc in todo.desc.iter().rev() {
            ui.add(Label::new(format!("{}", desc.created)).selectable(false));
            ui.add(Label::new(&desc.description).selectable(false));
            ui.separator();
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _: &mut eframe::Frame) {
        self.top_menu(ui);

        self.bottom_panel(ui);

        CentralPanel::default().show_inside(ui, |ui| {
            self.todo_sidebar(ui);

            if let Some(state) = self.state {
                match state {
                    State::SwitchProject(to_proj_id, _to_todo_id) => {
                        self.active_proj = Some(to_proj_id);
                        self.state = Some(State::Home(None));
                    }
                    State::ModalNewProject => {
                        self.new_project(ui);
                    }
                    State::ModalNewTodo(proj_id) => {
                        self.new_todo(ui, proj_id);
                    }
                    State::MarkTodoForDeletion(proj_id, todo_id) => {
                        self.confirm_todo_deletion(ui, proj_id, todo_id);
                    }
                    State::MarkProjectForDeletion(proj_id) => {
                        self.confirm_project_deletion(ui, proj_id);
                        if self.state.is_none() {
                            self.active_proj = None;
                        }
                    }
                    State::SwitchTodo(proj_id, todo_id) => {
                        self.descriptions_central(ui, proj_id, todo_id);
                    }
                    State::ModalNewDesc(proj_id, todo_id) => {
                        self.new_desc(ui, proj_id, todo_id);
                    }
                    State::Home(Some(todo_id)) => {
                        if let Some(active_proj) = self.active_proj {
                            let proj = self.projects.get(&active_proj).unwrap();
                            if let Some((todo_id, _)) = proj.todos.get_key_value(&todo_id) {
                                self.descriptions_central(ui, active_proj, *todo_id);
                            }
                        }
                    }
                    State::Home(None) => {}
                }
            }
        });
    }
}
