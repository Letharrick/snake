use std::convert::TryInto;

use bracket_terminal::prelude::{
    BTerm,
    BTermBuilder,
    BError,
    VirtualKeyCode,
    GameState,
    Point,
    RGB
};

use rand;
use rand::seq::SliceRandom;
use rand::rngs::ThreadRng;

use web_sys::Performance;

use crate::object::{Object, Obj};
use crate::snake::Snake;
use crate::direction::Direction;

#[cfg(not(target_arch = "wasm32"))]
use bracket_terminal::prelude::{INPUT, BEvent};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(not(target_arch = "wasm32"))]
pub type Timestamp = Instant;
#[cfg(target_arch = "wasm32")]
pub type Timestamp = f64;


pub struct Game {
    #[cfg(target_arch = "wasm32")]
    time: web_sys::Performance,
    rng: ThreadRng,
    snake: Snake,
    fruit: Object,
    score: usize,
    game_over: bool,
    paused: bool,
    previous_snake_update_time: Timestamp,
}

impl Game {
    pub const TITLE: &'static str = "Snake";

    pub const FRUIT_GLYPH: char = '*';
    pub const FRUIT_COLOUR: RGB = RGB {r: 1.0, g: 0.5, b: 0.5};
    pub const BACKGROUND_COLOUR: RGB = RGB {r: 0.175, g: 0.2, b: 0.225};

    pub const TILE_DIMENSIONS: (u32, u32) = (25, 25);
    pub const MAP_DIMENSIONS: (u32, u32) = (25, 25);

    pub const MAP_CENTRE: (u32, u32) = (
        Self::MAP_DIMENSIONS.0 / 2,
        Self::MAP_DIMENSIONS.1 / 2
    );

    pub const FRAMES_PER_SECOND: f32 = 60.0;
    pub const SLITHERS_PER_SECOND: u32 = 15;

    pub fn new() -> Self {
        // Attributes for the WASM version of the game
        let time: Performance;
        let previous_snake_update_time: f64;

        #[cfg(target_arch = "wasm32")]
        {
            time = web_sys::window().unwrap().performance().unwrap();
            previous_snake_update_time = time.now();
        }

        let mut game = Self {
            rng: rand::thread_rng(),
            snake: Snake::default(),
            fruit: Object::new((-1, -1).into(), Self::FRUIT_GLYPH, Self::FRUIT_COLOUR), // Initally positioned outside of map
            #[cfg(not(target_arch = "wasm32"))]
            previous_snake_update_time: Instant::now(),
            #[cfg(target_arch = "wasm32")]
            time,
            #[cfg(target_arch = "wasm32")]
            previous_snake_update_time,
            score: 0,
            game_over: false,
            paused: false
        };

        game.spawn_fruit();

        game
    }

    pub fn run(self) -> BError {
        // Build application
        let mut ctx = BTermBuilder::simple(Self::MAP_DIMENSIONS.0, Self::MAP_DIMENSIONS.1).expect("Failed to construct applciation builder")
            .with_title(Self::TITLE)
            .with_tile_dimensions(Self::TILE_DIMENSIONS.0, Self::TILE_DIMENSIONS.1)
            .with_fps_cap(Self::FRAMES_PER_SECOND)
            .with_advanced_input(true)
            .build().expect("Failed to build application context");

        ctx.with_post_scanlines(true);

        // Run game loop
        bracket_terminal::prelude::main_loop(ctx, self)
    }

    pub fn reset(&mut self) {
        self.snake = Snake::default();
        self.spawn_fruit();
        #[cfg(target_arch = "wasm32")]
        {
            self.previous_snake_update_time = self.time.now();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.previous_snake_update_time = Instant::now();
        }
        self.score = 0;
        self.game_over = false;
    }
    
    fn spawn_fruit(&mut self) {
        let spawn_locations = self.get_empty_points();

        self.fruit.position = *spawn_locations.choose(&mut self.rng).expect("Failed to spawn fruit");
    }

    fn update_snake(&mut self) {
        let update_delta;

        #[cfg(target_arch = "wasm32")]
        {
            update_delta = (self.time.now() - self.previous_snake_update_time) / 1000.0;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            update_delta = self.previous_snake_update_time.elapsed().as_secs_f64();
        }

        if (!self.snake.alive || !self.game_over) && update_delta > 1.0 / Self::SLITHERS_PER_SECOND as f64 {
            self.snake.update();

            #[cfg(target_arch = "wasm32")]
            {
                self.previous_snake_update_time = self.time.now();
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                self.previous_snake_update_time = Instant::now();
            }
        }
    }

    fn get_empty_points(&self) -> Vec<Point> {
        let mut empty_points = Vec::<Point>::default();
        let mut snake_segment_points = self.snake.iter().map(|cell| cell.position);
        
        for y in 0..Self::MAP_DIMENSIONS.1 {
            for x in 0..Self::MAP_DIMENSIONS.0 {
                let point = Into::<Point>::into((x as f32, y as f32));

                if !snake_segment_points.any(|p| p == point) && self.fruit.position != point {
                    empty_points.push(point)
                }
            }
        }

        empty_points
    }

    fn execute_input(&mut self, key_code: VirtualKeyCode) {
        if !self.game_over {
            match key_code {
                VirtualKeyCode::W | VirtualKeyCode::A |
                VirtualKeyCode::S | VirtualKeyCode::D |
                VirtualKeyCode::Up | VirtualKeyCode::Down |
                VirtualKeyCode::Left | VirtualKeyCode::Right => if self.snake.alive && !self.paused {
                    self.snake.set_direction(
                        TryInto::<Direction>::try_into(key_code).unwrap() // Change snake direction
                    )
                },
                VirtualKeyCode::Escape | VirtualKeyCode::P => {
                    self.paused = !self.paused
                }
                _ => {}
            }
        } else if key_code == VirtualKeyCode::R {
            self.reset();
        }
    }

    fn handle_input(&mut self, ctx: &mut BTerm) {
        #[cfg(target_arch = "wasm32")]
        {
            match ctx.key {
                Some(key_code) => self.execute_input(key_code),
                None => {}
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            INPUT.lock().for_each_message(|event| {
                match event {
                    BEvent::KeyboardInput {key, pressed: true, ..} => self.execute_input(key),
                    BEvent::CloseRequested => ctx.quit(),
                    _ => { }
                }
            });
        }
    }

    fn handle_logic(&mut self) {
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
        self.update_snake();
    }

    fn handle_rendering(&mut self, ctx: &mut BTerm) {
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
                ctx.print_color_centered_at(Self::MAP_CENTRE.0, Self::MAP_CENTRE.1 + 3, bracket_terminal::prelude::WHITE, Self::BACKGROUND_COLOUR, "[R] Restart");
            } else { // If the game is not over, continue rendering the fruit
                self.fruit.render(ctx);
            }
        }
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

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}