use args::Args;
use bevy::prelude::*;
use clap::Parser;
use settings::{read_settings, Settings};

mod protocol;
mod args;
mod settings;
mod apps;

fn main() {
    let args = Args::parse();
    let settings_str = include_str!("../assets/settings.ron");
    let settings = read_settings::<Settings>(settings_str);

    let mut apps = 
}
