mod app;
mod camera;
mod config;
mod geom;
mod material;
mod obj;
mod ray;
mod style;
mod texture;
mod vec;

use app::AppModel;

use ray::Ray;
use vec::*;

use iced::{Application, Settings};

pub fn main() {
    AppModel::run(Settings::default());
}
