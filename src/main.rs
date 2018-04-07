#![feature(slice_patterns)]

extern crate ggez;

mod path;

use ggez::{
  event::{self, MouseState, MouseButton},
  graphics::{self, Point2, DrawMode},
  GameResult,
  Context,
};

use path::{
  Path,
  Dir,
  Pos,
};

const GRID_SIZE: (i16, i16) = (30, 20);
const GRID_CELL_SIZE: i16 = 32;

const SCREEN_SIZE: (u32, u32) = (
  GRID_SIZE.0 as u32 * GRID_CELL_SIZE as u32,
  GRID_SIZE.1 as u32 * GRID_CELL_SIZE as u32,
);

struct GameState {
  mouse_pos: Pos,
  path: Option<Path>,
}

impl GameState {
  pub fn new() -> Self {
    GameState {
      mouse_pos: Pos(0, 0),
      path: None,
    }
  }
}

fn snap_to_grid(pos: Pos) -> Pos {
  let gs = GRID_CELL_SIZE as f32;
  let pos = (pos.0 as f32, pos.1 as f32);

  // tile offset
  let off = (pos.0 % gs, pos.1 % gs);
  // grid offset
  let (rx, ry) = (pos.0 - off.0, pos.1 - off.1);
  // relative offset
  let (x, y) = (off.0 / gs, off.1 / gs);

  let res = match (x > y, x + y < 1.) {
    (true, true) => (rx + gs / 2., ry),
    (true, false) => (rx + gs, ry + gs / 2.),
    (false, true) => (rx, ry + gs / 2.),
    (false, false) => (rx + gs / 2., ry + gs),
  };

  Pos(res.0 as i32, res.1 as i32)
}

impl event::EventHandler for GameState {
  fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
    Ok(())
  }

  fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
    graphics::clear(ctx);

    // draw a grid
    graphics::set_color(ctx, [0.0, 0.0, 0.0, 1.0].into())?;

    for i in 0..GRID_SIZE.0 {
      let x: f32 = (i * GRID_CELL_SIZE) as f32;
      let y: f32 = SCREEN_SIZE.1 as f32;

      graphics::line(ctx, &[Point2::new(x, 0.), Point2::new(x, y)], 1.)?;
    }
    for i in 0..GRID_SIZE.0 {
      let x: f32 = SCREEN_SIZE.0 as f32;
      let y: f32 = (i * GRID_CELL_SIZE) as f32;

      graphics::line(ctx, &[Point2::new(0., y), Point2::new(x, y)], 1.)?;
    }

    // draw the path
    if let Some(ref path) = self.path {
      path.draw(ctx)?;
    }

    // draw the mouse pos
    graphics::set_color(ctx, [1.0, 0.0, 1.0, 1.0].into())?;
    graphics::circle(ctx, DrawMode::Line(2.), Point2::new(self.mouse_pos.0 as f32, self.mouse_pos.1 as f32), 8., 0.1)?;

    // finish up
    graphics::present(ctx);
    ggez::timer::yield_now();

    Ok(())
  }

  fn mouse_button_down_event(
    &mut self,
    _ctx: &mut Context,
    button: MouseButton,
    mx: i32,
    my: i32,
  ) {
    let Pos(x, y) = self.mouse_pos;

    match button {
      MouseButton::Left => {
        // try add path node / start new path

        match self.path {
          Some(ref mut path) => { path.push(); }
          None => {
            let is_x = x % GRID_CELL_SIZE as i32 == 0;
            self.path = Some(Path::new(Pos(x, y), if is_x {
              if mx > x { Dir::Right } else { Dir::Left }
            } else {
              if my > y { Dir::Up } else { Dir::Down }
            }));
          }
        };

      },

      MouseButton::Right => {
        // try to find a path

        if let Some(ref mut path) = self.path {
          path.add_path(Pos(x, y));
        }
      },

      _ => {}
    };
  }

  fn mouse_motion_event(
    &mut self,
    _ctx: &mut Context,
    _state: MouseState,
    x: i32,
    y: i32,
    _: i32,
    _: i32,
  ) {
    let snap = snap_to_grid(Pos(x, y));

    if snap == self.mouse_pos {
      return;
    }

    self.mouse_pos = snap;

    if let Some(ref mut path) = self.path {
      path.add_path(snap);
    }
  }
}

fn main() {
  let ctx = &mut ggez::ContextBuilder::new("train_thing", "Leah")
      .window_setup(ggez::conf::WindowSetup::default().title("Trains!"))
      .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
      .build().expect("Failed to build ggez context");

  graphics::set_background_color(ctx, [1.0, 1.0, 1.0, 1.0].into());

  let state = &mut GameState::new();

  match event::run(ctx, state) {
    // If we encounter an error, we print it before exiting
    Err(e) => println!("Error encountered running game: {}", e),
    // And if not, we print a message saying we ran cleanly. Hooray!
    Ok(_) => println!("Game exited cleanly!")
  }
}
