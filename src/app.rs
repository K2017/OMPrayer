use crate::vec::*;
use rand::prelude::*;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;

use crate::config::UserConfig;
use crate::geom::*;
use iced::{
    button, scrollable, Align, Application, Button, Column, Command, Container, Element,
    HorizontalAlignment, Image, Length, Row, Scrollable, Space, Text,
};
use nfd::Response;
use tempfile::NamedTempFile;

use crate::{camera, style};
use names::{Generator, Name};
use tinyfiledialogs::{MessageBoxIcon, YesNo};

extern crate names;
extern crate nfd;
extern crate tinyfiledialogs;

#[derive(Default)]
pub struct AppModel {
    result: Vec<u8>,
    image: Option<iced::image::Handle>,
    temp_image_path: PathBuf,
    config: Option<UserConfig>,
    config_path: Option<PathBuf>,
    state: AppState,

    rand_adj: String,

    chooser_button: button::State,
    tracer_button: button::State,
    save_button: button::State,
    quit_button: button::State,

    scroll_state: scrollable::State,
}

#[derive(Debug, Clone, Copy)]
enum AppState {
    Ready,
    Rendering,
    Done,
}

impl Default for AppState {
    fn default() -> Self {
        Self::Ready
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    TraceError,
}

#[derive(Debug, Clone)]
pub enum Message {
    Done(Result<Vec<u8>, Error>),
    ChooseConfig,
    Trace,
    SaveImage,
    Quit,
}

impl Application for AppModel {
    type Executor = iced::executor::Default;
    type Message = Message;

    fn new() -> (Self, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Prayer")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        let mut command = Command::none();
        match message {
            Message::ChooseConfig => {
                let response = nfd::open_file_dialog(Some("toml"), None).unwrap_or_else(|e| {
                    panic!(e);
                });

                match response {
                    Response::Okay(path) => {
                        let path = PathBuf::from(path);
                        self.config_path = Some(path);
                        if let Some(path) = self.config_path.as_ref() {
                            let result = UserConfig::from_file(path);
                            match result {
                                Ok(config) => {
                                    self.config = Some(config);
                                }
                                Err(e) => {
                                    self.config = None;
                                    tinyfiledialogs::message_box_ok(
                                        "Configuration",
                                        format!("Error in configuration file:\n {}", e).as_str(),
                                        MessageBoxIcon::Info,
                                    );
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Message::Trace => {
                self.state = AppState::Rendering;
                if let Some(config) = self.config.as_ref().cloned() {
                    command = Command::perform(trace_main(config), Message::Done);
                }
            }
            Message::Done(Ok(buffer)) => {
                let config = self.config.as_ref().unwrap();
                self.result = buffer;
                let temp_file = NamedTempFile::new().unwrap().path().with_extension("png");
                self.temp_image_path = temp_file;
                image::save_buffer(
                    &self.temp_image_path,
                    &self.result,
                    config.params.resolution.x,
                    config.params.resolution.y,
                    image::RGB(8),
                )
                .unwrap();
                self.image = Some(iced::image::Handle::from_path(&self.temp_image_path));

                let mut gen = Generator::default(Name::Plain);
                let random_adj_noun = gen.next().unwrap();
                let words: Vec<&str> = random_adj_noun.as_str().split("-").collect();

                self.rand_adj = String::from(words[0]);

                self.state = AppState::Done;
            }
            Message::Done(Err(e)) => {
                tinyfiledialogs::message_box_ok(
                    "Configuration",
                    "Can't start tracing without a config file!",
                    MessageBoxIcon::Info,
                );
            }
            Message::SaveImage => {
                let response = nfd::open_save_dialog(Some("png"), None).unwrap_or_else(|e| {
                    panic!(e);
                });

                match response {
                    Response::Okay(path) => {
                        let _result = fs::copy(&self.temp_image_path, PathBuf::from(path))
                            .unwrap_or_else(|e| {
                                tinyfiledialogs::message_box_ok(
                                    "Error",
                                    format!("Image could not be saved: {}", e).as_str(),
                                    MessageBoxIcon::Error,
                                );
                                0
                            });
                    }
                    _ => {}
                }
            }
            Message::Quit => {
                let choice = tinyfiledialogs::message_box_yes_no(
                    "Quit",
                    "Are you sure?",
                    MessageBoxIcon::Question,
                    YesNo::No,
                );
                match choice {
                    YesNo::Yes => std::process::exit(0),
                    _ => {}
                }
            }
        }

        command
    }

    fn view(&mut self) -> Element<Message> {
        let mut main_view = Column::new();
        let mut scrollable = Scrollable::new(&mut self.scroll_state);
        let config_button =
            button(&mut self.chooser_button, "Choose config...").on_press(Message::ChooseConfig);
        let trace_button = button(&mut self.tracer_button, "Trace").on_press(Message::Trace);
        let save_button = button(&mut self.save_button, "Save Image")
            .style(style::Button::Secondary)
            .on_press(Message::SaveImage);
        let quit_button = button(&mut self.quit_button, "Quit")
            .style(style::Button::Destructive)
            .on_press(Message::Quit);

        let mut path_label = Text::new("");
        if let Some(path) = self.config_path.as_ref() {
            if let Some(_) = self.config {
                path_label = Text::new(path.to_str().unwrap());
            }
        }

        let mut menu_bar = Row::new();
        menu_bar = menu_bar
            .width(Length::Fill)
            .align_items(Align::Center)
            .push(config_button)
            .push(trace_button)
            .push(path_label)
            .push(Space::with_width(Length::Fill))
            .push(save_button)
            .push(quit_button);

        main_view = main_view.push(menu_bar);

        let mut container = Column::new();
        match self.state {
            AppState::Ready => {}
            AppState::Rendering => {}
            AppState::Done => {
                let label = Text::new(self.rand_adj.to_owned() + "!");

                if let Some(image) = self.image.as_ref() {
                    let img = Image::new(image.clone());
                    let img_container = Container::new(img);
                    container = container
                        .width(Length::Fill)
                        .align_items(Align::Center)
                        .push(label)
                        .push(img_container);
                }
            }
        };

        scrollable = scrollable.width(Length::Fill).push(container);
        main_view
            .push(Container::new(scrollable).height(Length::Fill))
            .into()
    }
}

async fn trace_main(config: UserConfig) -> Result<Vec<u8>, Error> {
    let UserConfig { params, scene } = config;

    let w = params.resolution.x;
    let h = params.resolution.y;
    let camera = camera::Camera::looking_at(
        glm::vec3(0.0, 2.0, -5.0),
        glm::vec3(0.0, 0.0, 0.0),
        glm::vec3(0.0, 1.0, 0.0),
        80.0,
        w as f32 / h as f32,
    );

    let buffer: Vec<u8> = (0..w * h)
        .into_par_iter()
        .flat_map(|i| {
            let x = i % w;
            let y = i / w;
            let color = (0..params.samples)
                .into_par_iter()
                .map(|_| {
                    let mut rng = rand::thread_rng();
                    let rand: f32 = rng.gen();
                    let u = (x as f32 + rand) / w as f32;
                    let rand: f32 = rng.gen();
                    let v = (y as f32 + rand) / h as f32;
                    let ray = camera.ray_at(u, v);
                    trace(&ray, &scene, params.max_light_bounces)
                })
                .sum::<Vec3>()
                / params.samples as f32;
            let color = glm::vec3(1.0, 1.0, 1.0) - glm::exp(&(-color * params.exposure));
            vec![
                (color.x.max(0.0).min(1.0).powf(1.0 / params.gamma) * 255.99) as u8,
                (color.y.max(0.0).min(1.0).powf(1.0 / params.gamma) * 255.99) as u8,
                (color.z.max(0.0).min(1.0).powf(1.0 / params.gamma) * 255.99) as u8,
            ]
        })
        .collect::<Vec<_>>();

    Ok(buffer)
}

fn button<'a, Message>(state: &'a mut button::State, label: &str) -> Button<'a, Message> {
    Button::new(
        state,
        Text::new(label).horizontal_alignment(HorizontalAlignment::Center),
    )
    .style(style::Button::Primary)
    .padding(6)
    .min_width(60)
}
