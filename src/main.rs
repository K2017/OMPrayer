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
use std::path::PathBuf;
use vec::*;

use config::UserConfig;
use geom::*;
use iced::{button, Application, Button, Column, Command, Element, Settings, Text};
use ray::Ray;

#[derive(Default)]
struct AppModel {
    chooser_button: button::State,
    tracer_button: button::State,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ChooserPressed,
    TracePressed,
}

impl Application for AppModel {
    type Message = Message;

    fn new() -> (Self, Command<Message>) {
        (
            Self {
                chooser_button: button::State::new(),
                tracer_button: button::State::new(),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Prayer")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ChooserPressed => {
                // let buffer = trace_with_config(self.config.as_ref().unwrap());
            }
            Message::TracePressed => {}
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message> {
        Column::new()
            .push(
                Button::new(&mut self.chooser_button, Text::new("Choose config..."))
                    .on_press(Message::ChooserPressed),
            )
            .into()
    }
}

fn quit_with_usage() -> ! {
    eprintln!("Usage: prayer CONFIG [OUTPUT]");
    std::process::exit(1)
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
