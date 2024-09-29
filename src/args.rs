use bevy::prelude::Resource;
use clap::Parser;

#[derive(Parser, Debug, Resource, Clone)]
pub struct Args {
    /// runs the game in synctest mode
    #[clap(long)]
    pub synctest: bool,
}
