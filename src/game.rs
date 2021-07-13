use std::convert::TryInto;
use std::time::Instant;

use bracket_terminal::prelude::{
    INPUT,
    BTerm,
    BTermBuilder,
    BEvent,
    VirtualKeyCode,
    GameState,
    Point,
    RGB
};

use rand;
use rand::seq::SliceRandom;
use rand::rngs::ThreadRng;

use crate::object::{Object, Obj};
use crate::snake::Snake;
use crate::direction::Direction;

pub struct Game {
    rng: ThreadRng,
    snake: Snake,
    fruit: Object,
    previous_snake_update_time: Instant,
    score: usize,
    game_over: bool,
    paused: bool
}

impl Game {
    pub const TITLE: &'static str = "Snake";

    pub const TILE_DIMENSIONS: (u32, u32) = (24, 24);
    pub const MAP_DIMENSIONS: (u32, u32) = (25, 25);

    pub const MAP_CENTRE: (u32, u32) = (
        Self::MAP_DIMENSIONS.0 / 2,
        Self::MAP_DIMENSIONS.1 / 2
    );

    pub const FRUIT_GLYPH: char = '*';
    pub const FRUIT_COLOUR: RGB = RGB {r: 1.0, g: 0.5, b: 0.5};
    pub const BACKGROUND_COLOUR: RGB = RGB {r: 0.175, g: 0.2, b: 0.225};

    pub const FRAMES_PER_SECOND: f32 = 60.0;
    pub const SLITHERS_PER_SECOND: u32 = 10;
    
    pub const RESTART_COMMAND: (char, VirtualKeyCode) = ('R', VirtualKeyCode::R);
    pub const QUIT_COMMAND: (char, VirtualKeyCode) = ('Q', VirtualKeyCode::Q);

    pub fn new() -> Self {
        let mut game = Self {
            rng: rand::thread_rng(),
            snake: Snake::default(),
            fruit: Object::new((-1, -1).into(), Self::FRUIT_GLYPH, Self::FRUIT_COLOUR).into(), // Initally positioned outside of map
            previous_snake_update_time: Instant::now(),
            score: 0,
            game_over: false,
            paused: false
        };

        game.spawn_fruit();

        game
    }

    pub fn run(self) {
        // Build application
        let mut ctx = BTermBuilder::simple(Self::MAP_DIMENSIONS.0, Self::MAP_DIMENSIONS.1).expect("Failed to construct applciation builder")
            .with_title(Self::TITLE)
            .with_tile_dimensions(Self::TILE_DIMENSIONS.0, Self::TILE_DIMENSIONS.1)
            .with_fps_cap(Self::FRAMES_PER_SECOND)
            .with_advanced_input(true)
            .build().expect("Failed to build application context");

        // ctx.screen_burn_color((50, 50, 75).into());
        ctx.with_post_scanlines(true);

        // Run game loop
        bracket_terminal::prelude::main_loop(ctx, self).expect("fatal error");
    }

    pub fn handle_input(&mut self, ctx: &mut BTerm) {
        // Handle events
        INPUT.lock().for_each_message(|event| { match event {
            BEvent::KeyboardInput {key, pressed, ..} => {
                if !self.game_over {
                    match key {
                        VirtualKeyCode::W | VirtualKeyCode::A |
                        VirtualKeyCode::S | VirtualKeyCode::D |
                        VirtualKeyCode::Up | VirtualKeyCode::Down |
                        VirtualKeyCode::Left | VirtualKeyCode::Right => if self.snake.alive && !self.paused {
                            self.snake.set_direction(
                                TryInto::<Direction>::try_into(key).unwrap() // Change snake direction
                            )
                        },
                        VirtualKeyCode::Escape | VirtualKeyCode::P => if pressed { self.paused = !self.paused },
                        _ => {}
                    }
                } else if key == Self::RESTART_COMMAND.1 {
                    self.reset();
                } else if key == Self::QUIT_COMMAND.1 {
                    ctx.quitting = true;
                }
            },
            BEvent::CloseRequested => ctx.quitting = true, // Close button clicked
            _ => {}
        }});
    }

    pub fn handle_logic(&mut self) {
        // Check and store the status of the game
        if !self.game_over {
            let won = self.snake.len() as u32 == Self::MAP_DIMENSIONS.0 * Self::MAP_DIMENSIONS.1;
            let lost = !self.snake.alive;

            self.game_over = won || lost;
        }

        // If the game is not over, check if the snake collides with the fruit
        if !self.game_over {
            let snake_head = self.snake[0];

            // If the snake collides with the fruit, grow the snake and respawn the fruit
            if snake_head.position == self.fruit.position {
                self.score += 1;
                self.snake.grow();
                self.spawn_fruit(); // Must respawn the fruit after the snake grows
            }
        }

        // Update the snake (Slither and update its corner tiles)
        if (!self.game_over || (self.game_over && !self.snake.alive)) && self.previous_snake_update_time.elapsed().as_secs_f32() > 1.0 / Self::SLITHERS_PER_SECOND as f32 {
            self.snake.update();
            self.previous_snake_update_time = Instant::now();
        }
    }

    pub fn handle_rendering(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(Self::BACKGROUND_COLOUR);

        if self.paused {
            ctx.print_color_centered_at(Self::MAP_CENTRE.0, Self::MAP_CENTRE.1, bracket_terminal::prelude::WHITE, Self::BACKGROUND_COLOUR, "PAUSED".to_string());
        } else {
            self.snake.render(ctx);

            // If the game is over, print end-game information
            if self.game_over {
                ctx.print_color_centered_at(Self::MAP_CENTRE.0, Self::MAP_CENTRE.1 - 3, bracket_terminal::prelude::WHITE, Self::BACKGROUND_COLOUR, "GAME OVER".to_string());
                ctx.print_color_centered_at(Self::MAP_CENTRE.0, Self::MAP_CENTRE.1, bracket_terminal::prelude::WHITE, Self::BACKGROUND_COLOUR, if self.snake.alive {
                    "You won!".to_string()
                } else {
                    format!("Score: {}", self.score)
                });
                ctx.print_color_centered_at(Self::MAP_CENTRE.0, Self::MAP_CENTRE.1 + 3, bracket_terminal::prelude::WHITE, Self::BACKGROUND_COLOUR, format!("[{}] Restart [{}] Quit", Self::RESTART_COMMAND.0, Self::QUIT_COMMAND.0));
            } else { // If the game is not over, continue rendering the fruit
                self.fruit.render(ctx);
            }
        }
    }

    pub fn get_empty_points(&self) -> Vec<Point> {
        let mut empty_points = Vec::<Point>::default();
        let snake_segment_points = self.snake.iter().map(|cell| cell.position).collect::<Vec<Point>>();
        
        for y in 0..Self::MAP_DIMENSIONS.1 {
            for x in 0..Self::MAP_DIMENSIONS.0 {
                let point = Into::<Point>::into((x as f32, y as f32));

                if !snake_segment_points.contains(&point) && self.fruit.position != point {
                    empty_points.push(point)
                }
            }
        }

        empty_points
    }

    pub fn spawn_fruit(&mut self) {
        let spawn_locations = self.get_empty_points();

        self.fruit.position = *spawn_locations.choose(&mut self.rng).expect("Failed to spawn fruit");
    }

    pub fn reset(&mut self) {
        self.snake = Snake::default();
        self.spawn_fruit();
        self.previous_snake_update_time = Instant::now();
        self.score = 0;
        self.game_over = false;
    }
}

impl GameState for Game {
    fn tick(&mut self, ctx: &mut BTerm) {
        self.handle_input(ctx);
        if !self.paused {
            self.handle_logic();
        }
        self.handle_rendering(ctx);
    }
}