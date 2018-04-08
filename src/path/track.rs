use path::{Connection, Pos, Dir};
use super::GRID_CELL_SIZE;

use std::f32::consts::PI;

use ggez::{
  graphics::{self},
  GameResult,
  Context,
};

pub const STRT_LEN: i32 = GRID_CELL_SIZE as i32;

pub trait TrackPiece {
  fn start(&self) -> Connection;
  fn end(&self) -> Connection;

  fn len(&self) -> i32 {
    STRT_LEN
  }

  fn lerp(&self, perc: f32) -> Pos {
    let start = self.start().pos;
    let end = self.end().pos;

    let mut diff = end - start;
    diff.scale(perc);
    diff
  }

  fn draw(&self, ctx: &mut Context) -> GameResult<()> {
    graphics::line(ctx, &[self.start().pos.into(), self.end().pos.into()], 2.)
  }
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Straight {
  start: Connection,
  end: Connection,
}

impl Straight {
  pub fn new(start: Connection, end: Connection) -> Self {
    Straight {
      start,
      end,
    }
  }
}

impl TrackPiece for Straight {
  fn start(&self) -> Connection {
    self.start
  }
  fn end(&self) -> Connection {
    self.end
  }
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Diagonal {
  start: Connection,
  end: Connection,
}

use std::f32::consts::SQRT_2;
pub const DIAG_LEN: i32 = (SQRT_2 * GRID_CELL_SIZE as f32) as i32;

impl Diagonal {
  pub fn new(start: Connection, end: Connection) -> Self {
    Diagonal {
      start,
      end,
    }
  }
}

impl TrackPiece for Diagonal {
  fn start(&self) -> Connection {
    self.start
  }
  fn end(&self) -> Connection {
    self.end
  }

  fn len(&self) -> i32 {
    DIAG_LEN
  }
}

const TURN_RADIUS: f32 = 2.5 * GRID_CELL_SIZE as f32;
const TURN_ANGLE: f32 = 0.643501102924346923828125;
// 0.75_f32.atan();
pub const TURN_LENGTH: i32 = (TURN_ANGLE * TURN_RADIUS) as i32;

const TURN_DIVISIONS: i32 = 4;
const TURN_ANGLE_FRACT: f32 = TURN_ANGLE / TURN_DIVISIONS as f32;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Turn {
  start: Connection,
  end: Connection,
  center: Pos,
  base_ang: f32,
  dir: i8,
}

impl Turn {
  pub fn new(start: Connection, end: Connection) -> Self {
    let prev_pos = start.pos;
    let curr_pos = end.pos;

    let matc = |pos: Pos, center: Pos, turn: i8, ang: f32, reverse: bool| -> (Pos, i8, f32) {
      (Pos((pos.0 as f32 + center.0 as f32 * 2.5) as i32, (pos.1 as f32 + center.1 as f32 * 2.5) as i32), turn, ang * 2. * PI - if reverse { turn as f32 * TURN_ANGLE } else { 0. })
    };

    use self::Dir::*;

    let (center, dir, base_ang) = match (&start.dir, &end.dir) {
      (Up, UpLeft) => matc(prev_pos, Dir::Left.into(), 1, 0.0, false),
      (Up, UpRight) => matc(prev_pos, Dir::Right.into(), -1, 0.5, false),
      (UpRight, Up) => matc(curr_pos, Dir::Left.into(), 1, 0.0, true),
      (UpRight, Right) => matc(curr_pos, Dir::Down.into(), -1, 0.25, true),
      (Right, UpRight) => matc(prev_pos, Dir::Up.into(), 1, 0.75, false),
      (Right, DownRight) => matc(prev_pos, Dir::Down.into(), -1, 0.25, false),
      (DownRight, Right) => matc(curr_pos, Dir::Up.into(), 1, 0.75, true),
      (DownRight, Down) => matc(curr_pos, Dir::Left.into(), -1, 0.0, true),
      (Down, DownRight) => matc(prev_pos, Dir::Right.into(), 1, 0.5, false),
      (Down, DownLeft) => matc(prev_pos, Dir::Left.into(), -1, 0.0, false),
      (DownLeft, Down) => matc(curr_pos, Dir::Right.into(), 1, 0.5, true),
      (DownLeft, Left) => matc(curr_pos, Dir::Up.into(), -1, 0.75, true),
      (Left, DownLeft) => matc(prev_pos, Dir::Down.into(), 1, 0.25, false),
      (Left, UpLeft) => matc(prev_pos, Dir::Up.into(), -1, 0.75, false),
      (UpLeft, Left) => matc(curr_pos, Dir::Down.into(), 1, 0.25, true),
      (UpLeft, Up) => matc(curr_pos, Dir::Right.into(), -1, 0.5, true),

      (a, b) => {
        unreachable!("invalid turn {:?} to {:?}", a, b);
      }
    };

    Turn {
      start,
      end,
      dir,
      base_ang,
      center,
    }
  }
}

impl TrackPiece for Turn {
  fn start(&self) -> Connection {
    self.start
  }
  fn end(&self) -> Connection {
    self.end
  }
  fn len(&self) -> i32 {
    TURN_LENGTH
  }
  fn lerp(&self, perc: f32) -> Pos {
    let Pos(cx, cy) = self.center;
    let div = TURN_ANGLE * perc;

    Pos(cx + (TURN_RADIUS * (self.base_ang + div * self.dir as f32).cos()) as i32, cy + (TURN_RADIUS * (self.base_ang + div * self.dir as f32).sin()) as i32)
  }

  fn draw(&self, ctx: &mut Context) -> GameResult<()> {
    let Pos(cx, cy) = self.center;

    let mut points: Vec<Pos> = Vec::new();
    points.push(self.start.pos);

    let divs = (1..TURN_DIVISIONS).map(|e| e as f32 * TURN_ANGLE_FRACT).collect::<Vec<f32>>();

    for div in divs {
      points.push(Pos(cx + (TURN_RADIUS * (self.base_ang + div * self.dir as f32).cos()) as i32, cy + (TURN_RADIUS * (self.base_ang + div * self.dir as f32).sin()) as i32));
    }

    points.push(self.end.pos);
    for window in points.windows(2) {
      let (prev, curr) = (window[0], window[1]);
      graphics::line(ctx, &[prev.into(), curr.into()], 2.)?;
    }

//    graphics::circle(ctx, graphics::DrawMode::Fill, self.center.into(), 3., 0.2)?;

    Ok(())
  }
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum Track {
  Diag(Diagonal),
  Turn(Turn),
  Strt(Straight),
}

impl From<(Connection, Connection)> for Track {
  fn from((start, end): (Connection, Connection)) -> Self {
    use self::Dir::*;

    match (start.dir, end.dir) {
      (Up, Up) => Track::Strt(Straight::new(start, end)),
      (Up, UpLeft) => Track::Turn(Turn::new(start, end)),
      (Up, UpRight) => Track::Turn(Turn::new(start, end)),
      (UpRight, UpRight) => Track::Diag(Diagonal::new(start, end)),
      (UpRight, Up) => Track::Turn(Turn::new(start, end)),
      (UpRight, Right) => Track::Turn(Turn::new(start, end)),
      (Right, Right) => Track::Strt(Straight::new(start, end)),
      (Right, UpRight) => Track::Turn(Turn::new(start, end)),
      (Right, DownRight) => Track::Turn(Turn::new(start, end)),
      (DownRight, DownRight) => Track::Diag(Diagonal::new(start, end)),
      (DownRight, Right) => Track::Turn(Turn::new(start, end)),
      (DownRight, Down) => Track::Turn(Turn::new(start, end)),
      (Down, Down) => Track::Strt(Straight::new(start, end)),
      (Down, DownRight) => Track::Turn(Turn::new(start, end)),
      (Down, DownLeft) => Track::Turn(Turn::new(start, end)),
      (DownLeft, DownLeft) => Track::Diag(Diagonal::new(start, end)),
      (DownLeft, Down) => Track::Turn(Turn::new(start, end)),
      (DownLeft, Left) => Track::Turn(Turn::new(start, end)),
      (Left, Left) => Track::Strt(Straight::new(start, end)),
      (Left, DownLeft) => Track::Turn(Turn::new(start, end)),
      (Left, UpLeft) => Track::Turn(Turn::new(start, end)),
      (UpLeft, UpLeft) => Track::Diag(Diagonal::new(start, end)),
      (UpLeft, Left) => Track::Turn(Turn::new(start, end)),
      (UpLeft, Up) => Track::Turn(Turn::new(start, end)),

      _ => unreachable!("This is not a valid Track"),
    }
  }
}

impl TrackPiece for Track {
  fn start(&self) -> Connection {
    match self {
      Track::Turn(t) => t.start(),
      Track::Diag(t) => t.start(),
      Track::Strt(t) => t.start(),
    }
  }

  fn end(&self) -> Connection {
    match self {
      Track::Turn(t) => t.end(),
      Track::Diag(t) => t.end(),
      Track::Strt(t) => t.end(),
    }
  }

  fn len(&self) -> i32 {
    match self {
      Track::Turn(t) => t.len(),
      Track::Diag(t) => t.len(),
      Track::Strt(t) => t.len(),
    }
  }

  fn lerp(&self, perc: f32) -> Pos {
    match self {
      Track::Turn(t) => t.lerp(perc),
      Track::Diag(t) => t.lerp(perc),
      Track::Strt(t) => t.lerp(perc),
    }
  }

  fn draw(&self, ctx: &mut Context) -> GameResult<()> {
    match self {
      Track::Turn(t) => t.draw(ctx),
      Track::Diag(t) => t.draw(ctx),
      Track::Strt(t) => t.draw(ctx),
    }
  }
}
