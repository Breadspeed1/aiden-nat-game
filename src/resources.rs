use bevy::prelude::Resource;



#[derive(Debug, Resource)]
pub struct WindowScale(pub u32);

impl WindowScale {
    pub fn new() -> Self {
        Self(1)
    }

    pub fn increase(&mut self) -> u32 {
        self.0 += 1;
        self.0
    }

    pub fn decrease(&mut self) -> u32 {
        self.0 = 1.max(self.0 - 1);
        self.0
    }
}