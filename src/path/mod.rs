use ggez::{
  graphics::{self, Point2, DrawMode},
  GameResult,
  Context,
};

use std::f32::consts::PI;
use std::collections::{HashMap};

use super::{
  GRID_CELL_SIZE,
  SCREEN_SIZE,
};

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Pos(pub i32, pub i32);

impl Pos {
  pub fn to_float(&self) -> (f32, f32) {
    (self.0 as f32, self.1 as f32)
  }
}

impl Into<Point2> for Pos {
  fn into(self) -> Point2 {
    let (x, y) = self.to_float();
    Point2::new(x, y)
  }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub enum Dir {
  Up,
  UpRight,
  Right,
  DownRight,
  Down,
  DownLeft,
  Left,
  UpLeft,
}
//
//impl Dir {
//  fn opposite(&self) -> Dir {
//    use self::Dir::*;
//
//    match self {
//      Up => Down,
//      UpRight => DownLeft,
//      Right => Left,
//      DownRight => UpLeft,
//      Down => Up,
//      DownLeft => UpRight,
//      Left => Right,
//      UpLeft => DownRight,
//    }
//  }
//}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Connection {
  pos: Pos,
  dir: Dir,
}

impl Connection {
  pub fn new(pos: Pos, dir: Dir) -> Self {
    Connection {
      pos,
      dir,
    }
  }
}

fn conn(pos: (f32, f32), dir: Dir, dist: f32) -> (Connection, i32) {
  (Connection::new(Pos(pos.0 as i32, pos.1 as i32), dir), (dist * 10.) as i32)
}

pub fn gen_connections(pos: Pos, dir: Dir) -> Vec<(Connection, i32)> {
  let gs = GRID_CELL_SIZE as f32;
  let (x, y) = pos.to_float();

  let is_x = x % gs == 0.;

  use self::Dir::*;

  let diag = 0.5 * 2f32.sqrt();
  let straight = 1.0;
  let curve = 1.7;

  let conns = match dir {
    Right => vec![
      conn((x + 1.5 * gs, y - 0.5 * gs), DownRight, curve),
      conn((x + 1. * gs, y), Right, straight),
      conn((x + 1.5 * gs, y + 0.5 * gs), UpRight, curve),
    ],
    UpRight => vec![
      conn((x + 0.5 * gs, y + 0.5 * gs), UpRight, diag),
      if is_x {
        conn((x + 0.5 * gs, y + 1.5 * gs), Up, curve)
      } else {
        conn((x + 1.5 * gs, y + 0.5 * gs), Right, curve)
      },
    ],
    DownRight => vec![
      conn((x + 0.5 * gs, y - 0.5 * gs), DownRight, diag),
      if is_x {
        conn((x + 0.5 * gs, y - 1.5 * gs), Down, curve)
      } else {
        conn((x + 1.5 * gs, y - 0.5 * gs), Right, curve)
      },
    ],
    Up => vec![
      conn((x + 0.5 * gs, y + 1.5 * gs), UpRight, curve),
      conn((x, y + 1. * gs), Up, straight),
      conn((x - 0.5 * gs, y + 1.5 * gs), UpLeft, curve),
    ],
    Down => vec![
      conn((x - 0.5 * gs, y - 1.5 * gs), DownLeft, curve),
      conn((x, y - 1. * gs), Down, straight),
      conn((x + 0.5 * gs, y - 1.5 * gs), DownRight, curve),
    ],
    Left => vec![
      conn((x - 1.5 * gs, y + 0.5 * gs), UpLeft, curve),
      conn((x - 1. * gs, y), Left, straight),
      conn((x - 1.5 * gs, y - 0.5 * gs), DownLeft, curve),
    ],
    UpLeft => vec![
      conn((x - 0.5 * gs, y + 0.5 * gs), UpLeft, diag),
      if is_x {
        conn((x - 0.5 * gs, y + 1.5 * gs), Up, curve)
      } else {
        conn((x - 1.5 * gs, y + 0.5 * gs), Left, curve)
      },
    ],
    DownLeft => vec![
      conn((x - 0.5 * gs, y - 0.5 * gs), DownLeft, diag),
      if is_x {
        conn((x - 0.5 * gs, y - 1.5 * gs), Down, curve)
      } else {
        conn((x - 1.5 * gs, y - 0.5 * gs), Left, curve)
      },
    ],
  };

  conns.iter().filter(|(c,_)|
      c.pos.0 > 0 && c.pos.0 < SCREEN_SIZE.0 as i32 && c.pos.1 > 0 && c.pos.1 < SCREEN_SIZE.1 as i32
  ).map(|e| *e).collect()
}

// grid size, not screen size
#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Path {
  path: Vec<Connection>,
  poss_path: Option<Vec<Connection>>,
}

impl Path {
  pub fn new(start: Pos, dir: Dir) -> Self {
    Path {
      path: vec![conn((start.0 as f32, start.1 as f32), dir, 0.).0],
      poss_path: None,
    }
  }

  pub fn head(&self) -> &Connection {
    self.path.last().expect("Should always have at least one element")
  }

  pub fn push(&mut self) {
    if let Some(ref mut path) = self.poss_path {
      self.path.append(path);
    }

    self.poss_path = None;
  }

  fn draw_track(&self, ctx: &mut Context, prev: &Connection, curr: &Connection) -> GameResult<()> {
    let prev_pos = (prev.pos.0 as f32, prev.pos.1 as f32);
    let curr_pos = (curr.pos.0 as f32, curr.pos.1 as f32);

    let (lx, ly) = prev_pos;
    let (x, y) = curr_pos;

    if prev.dir == curr.dir {
      graphics::line(ctx, &[Point2::new(lx, ly), Point2::new(x, y)], 2.)?;
      return Ok(());
    }

    use self::Dir::*;

    let r = 2.5 * GRID_CELL_SIZE as f32;

    let full_angle = 0.75_f32.atan();
    let divisions = 4;
    let fract = full_angle / divisions as f32;

    let divs = (1..divisions).map(|e| e as f32 * fract).collect::<Vec<f32>>();

    let mut points: Vec<(f32, f32)> = vec![(prev.pos.0 as f32, prev.pos.1 as f32)];

    let matc = |pos: (f32, f32), center: (f32, f32), turn: f32, ang: f32, reverse: bool| -> (f32, f32, f32, f32, bool) {
      (pos.0 + center.0 * r, pos.1 + center.1 * r, turn, ang * 2. * PI, reverse)
    };

    let (cx, cy, turn, ang, reverse) = match (&prev.dir, &curr.dir) {
      (Up, UpLeft) => matc(prev_pos, (-1., 0.), 1., 0.0, false),
      (Up, UpRight) => matc(prev_pos, (1., 0.), -1., 0.5, false),
      (UpRight, Up) => matc(curr_pos, (-1., 0.), -1., 0.0, true),
      (UpRight, Right) => matc(curr_pos, (0., -1.), 1., 0.25, true),
      (Right, UpRight) => matc(prev_pos, (0., 1.), 1., 0.75, false),
      (Right, DownRight) => matc(prev_pos, (0., -1.), -1., 0.25, false),
      (DownRight, Right) => matc(curr_pos, (0., 1.), -1., 0.75, true),
      (DownRight, Down) => matc(curr_pos, (-1., 0.), 1., 0.0, true),
      (Down, DownRight) => matc(prev_pos, (1., 0.), 1., 0.5, false),
      (Down, DownLeft) => matc(prev_pos, (-1., 0.), -1., 0.0, false),
      (DownLeft, Down) => matc(curr_pos, (1., 0.), -1., 0.5, true),
      (DownLeft, Left) => matc(curr_pos, (0., 1.), 1., 0.75, true),
      (Left, DownLeft) => matc(prev_pos, (0., -1.), 1., 0.25, false),
      (Left, UpLeft) => matc(prev_pos, (0., 1.), -1., 0.75, false),
      (UpLeft, Left) => matc(curr_pos, (0., -1.), -1., 0.25, true),
      (UpLeft, Up) => matc(curr_pos, (1., 0.), 1., 0.5, true),

      (a, b) => {
        println!("invalid turn {:?} to {:?}", a, b);
        return Ok(());
      } // invalid but idc
    };

    {
      let fun = |div: &f32| points.push((cx + r * (ang + div * turn).cos(), cy + r * (ang + div * turn).sin()));

      if reverse {
        divs.iter().rev().for_each(fun);
      } else {
        divs.iter().for_each(fun);
      }
    }

    points.push(curr_pos.clone());

    for window in points.windows(2) {
      let (prev, curr) = (window[0], window[1]);
      let (lx, ly) = prev;
      let (x, y) = curr;
      graphics::line(ctx, &[Point2::new(lx, ly), Point2::new(x, y)], 2.)?;
    }

//    graphics::circle(ctx, DrawMode::Fill, Point2::new(cx, cy), 3., 0.2)?;

    Ok(())
  }

  pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
    // draw path
    graphics::set_color(ctx, [0.0, 0.0, 1.0, 1.0].into())?;

    for window in self.path.windows(2) {
      let (prev, curr) = (&window[0], &window[1]);
      self.draw_track(ctx, &prev, &curr)?;
    }

    graphics::set_color(ctx, [0.0, 0.7, 0.2, 1.0].into())?;
    if let Some(ref path) = self.poss_path {
      for window in path.windows(2) {
        let (prev, curr) = (&window[0], &window[1]);
        self.draw_track(ctx, &prev, &curr)?;
      }
    }

    // current pos
    graphics::set_color(ctx, [1.0, 0.0, 0.0, 1.0].into())?;
    let pos = self.head().pos;
    graphics::circle(ctx, DrawMode::Fill, pos.into(), 4., 0.2)?;

    Ok(())
  }

  fn estimate(from: &Connection, to: &Pos) -> i32 {
    ((from.pos.0 - to.0).abs() + (from.pos.1 - to.0).abs()) * 10
  }

  pub fn add_path(&mut self, to: Pos) {
    let path = self.find_path(to);

//    println!("path: {:?}", path);

    self.poss_path = path;
  }

  pub fn find_path(&self, to: Pos) -> Option<Vec<Connection>> {
    let mut closed: HashMap<Connection, Node> = HashMap::new();
    let mut open: HashMap<Connection, Node> = HashMap::new();

    let mut from = HashMap::new();

    let head = self.head().clone();

    let start = Node {
      conn: head,
      g_score: 0i32,
      f_score: Path::estimate(&head, &to),
    };

    open.insert(head, start);

    while open.len() > 0 {
      let curr: Node = open.values().fold(None, |acc, v| match acc {
        None => Some(v),
        Some(c) => if c.g_score <= v.g_score { Some(c) } else { Some(v) }
      }).expect("should have at least one element at this point").clone();

      if curr.conn.pos == to {
        let mut curr = curr.conn;
        let mut total: Vec<Connection> = vec![curr];

        while let Some(par) = from.get(&curr) {
          curr = *par;
          total.push(curr);
        }

        total.reverse();

        return Some(total);
      }

      open.remove(&curr.conn);
      closed.insert(curr.conn, curr);

      for (next, dist) in gen_connections(curr.conn.pos, curr.conn.dir) {
        if closed.contains_key(&next) {
          continue;
        }

        let total_g = curr.g_score + dist;

        let mut n_node = open.entry(next).or_insert_with(|| {
          from.insert(next, curr.conn);
          Node {
            conn: next,
            g_score: total_g,
            f_score: total_g + Path::estimate(&next, &to),
          }
        });

        if n_node.g_score <= total_g {
          continue;
        }

        from.insert(next, curr.conn);

        n_node.g_score = total_g;
        n_node.g_score = total_g + Path::estimate(&next, &to);
      }
    }

    return None;
  }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
struct Node {
  conn: Connection,
  g_score: i32,
  f_score: i32,
}
