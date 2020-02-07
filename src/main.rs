mod camera;
mod config;
mod geom;
mod material;
mod obj;
mod ray;
mod texture;
mod vec;

use rand::prelude::*;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use vec::*;

use config::UserConfig;
use geom::*;
use iced::{
    button, Align, Application, Button, Column, Command, Container, Element, Image, Length, Row,
    Settings, Text,
};
use nfd::Response;
use ray::Ray;
use tempfile::NamedTempFile;

use names::{Generator, Name};
use tinyfiledialogs::{MessageBoxIcon, YesNo};

extern crate names;
extern crate nfd;
extern crate tinyfiledialogs;

#[derive(Default)]
struct AppModel {
    result: Vec<u8>,
    image: Option<iced::image::Handle>,
    temp_image_path: PathBuf,
    config: Option<UserConfig>,
    state: AppState,

    rand_adj: String,

    chooser_button: button::State,
    tracer_button: button::State,
    save_button: button::State,
    quit_button: button::State,
    again_button: button::State,
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

#[derive(Debug, Clone, Copy)]
enum Message {
    ChooseConfig,
    Trace,
    SaveImage,
    Quit,
    GoAgain,
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
        match message {
            Message::ChooseConfig => {
                let response = nfd::open_file_dialog(Some("toml"), None).unwrap_or_else(|e| {
                    panic!(e);
                });

                match response {
                    Response::Okay(path) => {
                        self.config = Some(UserConfig::from_file(&PathBuf::from(path)).unwrap());
                    }
                    _ => {}
                }
            }
            Message::Trace => {
                if let Some(config) = self.config.as_ref() {
                    self.state = AppState::Rendering;

                    self.result = trace_with_config(config);

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
            Message::GoAgain => {
                self.state = AppState::Ready;
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        match self.state {
            AppState::Ready => Column::new()
                .push(
                    Button::new(&mut self.chooser_button, Text::new("Choose config..."))
                        .on_press(Message::ChooseConfig),
                )
                .push(
                    Button::new(&mut self.tracer_button, Text::new("Trace"))
                        .on_press(Message::Trace),
                )
                .into(),
            AppState::Rendering => Column::new()
                .push(
                    Button::new(&mut self.chooser_button, Text::new("Choose config..."))
                        .on_press(Message::ChooseConfig),
                )
                .into(),
            AppState::Done => {
                let label = Text::new(self.rand_adj.to_owned() + "!");
                let save = Button::new(&mut self.save_button, Text::new("Save Image"))
                    .on_press(Message::SaveImage);
                let quit =
                    Button::new(&mut self.quit_button, Text::new("Quit")).on_press(Message::Quit);
                let again = Button::new(&mut self.again_button, Text::new("Trace another..."))
                    .on_press(Message::GoAgain);
                let bottom_bar = Column::new()
                    .padding(10)
                    .push(Row::new().push(save).push(again).push(quit));
                if let Some(image) = self.image.as_ref() {
                    let img = Image::new(image.clone());
                    let img_container = Container::new(img)
                        .width(Length::Units(600))
                        .height(Length::Units(600));
                    return Column::new()
                        .align_items(Align::Center)
                        .push(label)
                        .push(img_container)
                        .push(bottom_bar)
                        .into();
                }

                Column::new().push(label).into()
            }
        }
    }
}

fn trace_with_config(config: &UserConfig) -> Vec<u8> {
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

    (0..w * h)
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
        .collect::<Vec<_>>()
}

pub fn main() {
    /*
    let usr_config = UserConfig::from_file(&config).unwrap_or_else(|e| {
        eprintln!("Error parsing {}: {}", config.display(), e);
        std::process::exit(1)
    });
    */
    AppModel::run(Settings::default());

    // image::save_buffer(&image, &buffer, w, h, image::RGB(8)).unwrap()
}
