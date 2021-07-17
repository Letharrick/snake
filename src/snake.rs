use bracket_terminal::prelude::{
    BTerm,
    Point,
    RGB
};

use std::cmp::Ordering;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};

use crate::game::Game;
use crate::object::{Object, Obj};
use crate::direction::Direction;

pub struct Snake {
    body: VecDeque<Object>,
    direction: Direction,
    popped_tail: Option<Object>, // The tail of the snake prior to a successful movement. Used for extending the snake after a fruit is obtained
    requires_corner_update: bool, // For determining whether or not the glyphs of the corner segments of the snake need to be updated
    pub alive: bool
}

impl Snake {
    pub const STARTING_DIRECTIN: Direction = Direction::East;
    pub const STARTING_LENGTH: usize = 5;
    pub const HORIZONTAL_GLYPH: char = '═';
    pub const VERTICAL_GLYPH: char = '║';
    pub const CORNER_GLYPHS: (char, char, char, char) = (
        '╔', '╗',
        '╚', '╝',
    );
    pub const COLOUR: RGB = RGB {r: 0.5, g: 1.0, b: 0.5};
    pub const DEAD_COLOUR: RGB = RGB {r: 0.5, g: 0.5, b: 0.5};

    pub fn set_direction(&mut self, direction: Direction) {
        if self[0].position + Into::<Point>::into(direction) != self[1].position {
            self.direction = direction;
            self.requires_corner_update = true;
        }
    }

    pub fn grow(&mut self) {
        if let Some(tail) = self.popped_tail {
            self.push_back(tail);
        }
    }

    pub fn update_corner_glyphs(&mut self) {
        // Adjust glyphs of corners for when the snake turns
        if self.len() > 2 {
            let new_glyph = self[self.len() - 2].glyph;
            let head = self[0];
            let tail = self.back_mut().unwrap();

            // Straighten out tail if necessary
            if (new_glyph == Self::HORIZONTAL_GLYPH || new_glyph == Self::VERTICAL_GLYPH) &&
                tail.glyph != Self::HORIZONTAL_GLYPH && tail.glyph != Self::VERTICAL_GLYPH {
                tail.glyph = new_glyph;
            }

            if self.requires_corner_update {
                let neck_1 = self[2];
                let neck_0 = self.get_mut(1).unwrap();

                 if neck_1.position.x != neck_0.position.x {
                    match head.position.y.cmp(&neck_0.position.y) {
                        Ordering::Greater => neck_0.glyph = if neck_1.position.x > neck_0.position.x {
                            Self::CORNER_GLYPHS.0
                        } else {
                            Self::CORNER_GLYPHS.1
                        },
                        Ordering::Less => neck_0.glyph = if neck_1.position.x > neck_0.position.x {
                            Self::CORNER_GLYPHS.2
                        } else {
                            Self::CORNER_GLYPHS.3
                        },
                        _ => {}
                    };
                } else if neck_1.position.y != neck_0.position.y {
                    match head.position.x.cmp(&neck_0.position.x) {
                        Ordering::Greater => neck_0.glyph = if neck_1.position.y > neck_0.position.y {
                            Self::CORNER_GLYPHS.0
                        } else {
                            Self::CORNER_GLYPHS.2
                        },
                        Ordering::Less => neck_0.glyph = if neck_1.position.y > neck_0.position.y {
                            Self::CORNER_GLYPHS.1
                        } else {
                            Self::CORNER_GLYPHS.3
                        },
                        _ => {}
                    };
                }

                self.requires_corner_update = false;
            }
        }
    }
}

impl Obj for Snake {
    fn render(&self, ctx: &mut BTerm) {
        for segment in self.iter() {
            segment.render(ctx);
        }
    }

    fn update(&mut self) {
        if self.alive {
            let head = self[0];

            let out_of_bounds = 
                head.position.x < 0 || head.position.x >= Game::MAP_DIMENSIONS.0 as i32 ||
                head.position.y < 0 || head.position.y >= Game::MAP_DIMENSIONS.1 as i32;
            let self_collision = self.range(1..).map(|seg| seg.position).any(|point| point == head.position);

            self.alive = !self_collision && !out_of_bounds;

            if !self.alive {
                for segment in &mut self.body {
                    segment.colour = Self::DEAD_COLOUR;
                }
            }
        }

        if self.alive {
            let mut head = self[0];

            head.position = head.position + Into::<Point>::into(self.direction);
            head.glyph = match self.direction {
                Direction::North | Direction::South => Self::VERTICAL_GLYPH,
                Direction::East | Direction::West => Self::HORIZONTAL_GLYPH
            };

            self.popped_tail = self.pop_back();
            self.push_front(head);

            self.update_corner_glyphs();
        } else {
            self.pop_front();
        }
    }
}

impl Default for Snake {
    fn default() -> Self {
        let spawn_point =  Point::from((
            Game::MAP_CENTRE.0 as i32,
            Game::MAP_CENTRE.1 as i32
        ));

        let body_segment = Object::new(spawn_point, Self::HORIZONTAL_GLYPH, Self::COLOUR);
        let mut body = VecDeque::from(vec![body_segment; Self::STARTING_LENGTH - 1]);

        let mut head = body_segment;
        head.position = head.position + Into::<Point>::into(Self::STARTING_DIRECTIN);
        body.push_front(head);

        Self {
            body,
            direction: Self::STARTING_DIRECTIN,
            popped_tail: None,
            requires_corner_update: false,
            alive: true
        }
    }
}

impl Deref for Snake {
    type Target = VecDeque<Object>;

    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

impl DerefMut for Snake {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.body
    }
}