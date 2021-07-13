use bracket_terminal::prelude::{
    BTerm,
    Point,
    RGB,
};

use crate::game::Game;

pub trait Obj {
    fn render(&self, ctx: &mut BTerm);
    fn update(&mut self) {}
}

#[derive(Copy, Clone)]
pub struct Object {
    pub position: Point,
    pub glyph: char,
    pub colour: RGB,
}

impl Object {
    pub fn new(position: Point, glyph: char, colour: RGB) -> Self {
        Self {
            position,
            glyph,
            colour
        }
    }
}

impl Obj for Object {
    fn render(&self, ctx: &mut BTerm) {
        ctx.set(
            self.position.x, self.position.y,
            self.colour,
            Game::BACKGROUND_COLOUR,
            bracket_terminal::prelude::to_cp437(self.glyph)
        )
    }
}