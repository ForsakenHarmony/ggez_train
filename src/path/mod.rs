pub mod track;

use ggez::{
  graphics::{self, Point2, DrawMode},
  GameResult,
  Context,
};

use std::{
  collections::HashMap,
  ops::{
    Add,
    Sub,
  },
};

use super::{
  GRID_CELL_SIZE,
  SCREEN_SIZE,
};

use self::track::{TrackPiece, Track, TURN_LENGTH, DIAG_LEN, STRT_LEN};

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Hash)]
pub struct Pos(pub i32, pub i32);

impl Pos {
  pub fn to_float(&self) -> (f32, f32) {
    (self.0 as f32, self.1 as f32)
  }

  fn scale(&mut self, scl: f32) {
    self.0 = (self.0 as f32 * scl) as i32;
    self.1 = (self.1 as f32 * scl) as i32;
  }
}

impl From<Point2> for Pos {
  fn from(p: Point2) -> Self {
    Pos(p.x as i32, p.y as i32)
  }
}

impl Into<Point2> for Pos {
  fn into(self) -> Point2 {
    Point2::new(self.0 as f32, self.1 as f32)
  }
}

impl From<Dir> for Pos {
  fn from(p: Dir) -> Self {
    p.to_pos()
  }
}

//impl Into<Pos> for Dir {
//  fn into(self) -> Pos {
//    self.to_pos()
//  }
//}

impl Add for Pos {
  type Output = Pos;

  fn add(self, rhs: Pos) -> Pos {
    Pos(
      self.0 + rhs.0,
      self.1 + rhs.1,
    )
  }
}

impl Sub for Pos {
  type Output = Pos;

  fn sub(self, rhs: Pos) -> Pos {
    Pos(
      self.0 - rhs.0,
      self.1 - rhs.1,
    )
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

impl Dir {
  fn to_pos(&self) -> Pos {
    use self::Dir::*;

    let diag = (2f32.sqrt() * GRID_CELL_SIZE as f32) as i32;
    let strt = GRID_CELL_SIZE as i32;

    match &self {
      Up => Pos(0, strt),
      UpRight => Pos(diag, diag),
      Right => Pos(strt, 0),
      DownRight => Pos(diag, -diag),
      Down => Pos(0, -strt),
      DownLeft => Pos(-diag, -diag),
      Left => Pos(-strt, 0),
      UpLeft => Pos(-diag, diag),
    }
  }

  fn opposite(&self) -> Dir {
    use self::Dir::*;

    match self {
      Up => Down,
      UpRight => DownLeft,
      Right => Left,
      DownRight => UpLeft,
      Down => Up,
      DownLeft => UpRight,
      Left => Right,
      UpLeft => DownRight,
    }
  }
}

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

  fn gen_connections(&self) -> Vec<(Connection, i32)> {
    let start = *self;

    let gs = GRID_CELL_SIZE as f32;
    let (x, y) = start.pos.to_float();

    let is_x = x % gs == 0.;

    use self::Dir::*;

    let conn = |pos: (f32, f32), dir: Dir| {
      Connection::new(Pos(pos.0 as i32, pos.1 as i32), dir)
    };

    let conns = match start.dir {
      Right => vec![
        (conn((x + 1.5 * gs, y - 0.5 * gs), DownRight), TURN_LENGTH),
        (conn((x + 1. * gs, y), Right), STRT_LEN),
        (conn((x + 1.5 * gs, y + 0.5 * gs), UpRight), TURN_LENGTH),
      ],
      UpRight => vec![
        (conn((x + 0.5 * gs, y + 0.5 * gs), UpRight), DIAG_LEN),
        if is_x {
          (conn((x + 0.5 * gs, y + 1.5 * gs), Up), TURN_LENGTH)
        } else {
          (conn((x + 1.5 * gs, y + 0.5 * gs), Right), TURN_LENGTH)
        },
      ],
      DownRight => vec![
        (conn((x + 0.5 * gs, y - 0.5 * gs), DownRight), DIAG_LEN),
        if is_x {
          (conn((x + 0.5 * gs, y - 1.5 * gs), Down), TURN_LENGTH)
        } else {
          (conn((x + 1.5 * gs, y - 0.5 * gs), Right), TURN_LENGTH)
        },
      ],
      Up => vec![
        (conn((x + 0.5 * gs, y + 1.5 * gs), UpRight), TURN_LENGTH),
        (conn((x, y + 1. * gs), Up), STRT_LEN),
        (conn((x - 0.5 * gs, y + 1.5 * gs), UpLeft), TURN_LENGTH),
      ],
      Down => vec![
        (conn((x - 0.5 * gs, y - 1.5 * gs), DownLeft), TURN_LENGTH),
        (conn((x, y - 1. * gs), Down), STRT_LEN),
        (conn((x + 0.5 * gs, y - 1.5 * gs), DownRight), TURN_LENGTH),
      ],
      Left => vec![
        (conn((x - 1.5 * gs, y + 0.5 * gs), UpLeft), TURN_LENGTH),
        (conn((x - 1. * gs, y), Left), STRT_LEN),
        (conn((x - 1.5 * gs, y - 0.5 * gs), DownLeft), TURN_LENGTH),
      ],
      UpLeft => vec![
        (conn((x - 0.5 * gs, y + 0.5 * gs), UpLeft), DIAG_LEN),
        if is_x {
          (conn((x - 0.5 * gs, y + 1.5 * gs), Up), TURN_LENGTH)
        } else {
          (conn((x - 1.5 * gs, y + 0.5 * gs), Left), TURN_LENGTH)
        },
      ],
      DownLeft => vec![
        (conn((x - 0.5 * gs, y - 0.5 * gs), DownLeft), DIAG_LEN),
        if is_x {
          (conn((x - 0.5 * gs, y - 1.5 * gs), Down), TURN_LENGTH)
        } else {
          (conn((x - 1.5 * gs, y - 0.5 * gs), Left), TURN_LENGTH)
        },
      ],
    };

    conns.into_iter().filter(|(p, _)| {
      let Pos(x, y) = p.pos;
      x > 0 && x < SCREEN_SIZE.0 as i32 && y > 0 && y < SCREEN_SIZE.1 as i32
    }).collect()
  }
}

// grid size, not screen size
// #[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Path {
  start: Connection,
  path: Vec<Track>,
  poss_path: Option<Vec<Track>>,
}

impl Path {
  pub fn new(start: Pos, dir: Dir) -> Self {
    Path {
      start: Connection::new(start, dir),
      path: vec![],
      poss_path: None,
    }
  }

  pub fn push(&mut self) {
    if let Some(ref mut path) = self.poss_path {
      self.start = path.last().unwrap().end();
      self.path.append(path);
    }

    self.poss_path = None;
  }

  pub fn as_pieces(self) {}

  pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
    // draw path
    graphics::set_color(ctx, [0.0, 0.0, 1.0, 1.0].into())?;

    for track in self.path.iter() {
      track.draw(ctx)?;
    }

    graphics::set_color(ctx, [0.0, 0.7, 0.2, 1.0].into())?;
    if let Some(ref path) = self.poss_path {
      for track in path.iter() {
        track.draw(ctx)?;
      }
    }

    // current pos
    graphics::set_color(ctx, [1.0, 0.0, 0.0, 1.0].into())?;
    let pos = self.start.pos;
    graphics::circle(ctx, DrawMode::Fill, pos.into(), 4., 0.2)?;

    Ok(())
  }

  fn estimate(from: &Connection, to: &Pos) -> i32 {
    ((from.pos.0 - to.0).abs() + (from.pos.1 - to.0).abs()) * 10
  }

  pub fn add_path(&mut self, to: Pos) {
    let path = self.find_path(to);

    self.poss_path = match path {
      Some(path) => {
        Some(path.windows(2).map(|c| Track::from((c[0], c[1]))).collect::<Vec<Track>>())
      }
      None => None,
    };
  }

  pub fn find_path(&self, to: Pos) -> Option<Vec<Connection>> {
    let mut open: Vec<usize> = Vec::new();
    let mut closed: Vec<usize> = Vec::new();

    let mut nodes: Vec<Node> = Vec::new();

    let mut lookup: HashMap<Connection, usize> = HashMap::new();

    let mut children: Vec<usize> = Vec::new();

    let head = self.start;

    let start = Node {
      conn: head,
      g_score: 0i32,
      f_score: Path::estimate(&head, &to),
    };

    let mut count = 0;

    lookup.insert(start.conn, count);
    open.push(count);
    nodes.push(start);
    children.push(0);
    count += 1;

    while open.len() > 0 {
      let target: usize = open.iter().fold((0, i32::max_value()), |acc, i| {
        let node = nodes.get(*i).expect("nodes in the open list exist");
        if node.f_score < acc.1 { (*i, node.f_score) } else { acc }
      }).0;

      let node = nodes[target];

      if node.conn.pos == to {
        let mut target = target;

        let mut total = Vec::new();

        while target != 0 {
          total.push(target);
          target = children[target];
        }

        total.push(target);

        total.reverse();

        return Some(
          total
              .iter()
              .map(|i| nodes.get(*i).expect("all nodes in the children list should exist").conn)
              .collect()
        );
      }

      open.remove_item(&target);
      closed.push(target);

      for (conn, len) in node.conn.gen_connections() {
        let total_g = node.g_score + len * 10;

        if let Some(i) = lookup.get(&conn) {
          if !closed.contains(i) {
            let mut n_node = nodes.get_mut(*i).expect("nodes should exists if they are in lookup");

            if n_node.g_score <= total_g {
              continue;
            }

            let mut child = children.get_mut(*i).expect("a children entry should exist for all nodes");
            *child = target;

            n_node.g_score = total_g;
            n_node.f_score = total_g + Path::estimate(&n_node.conn, &to)
          }
          continue;
        }

        let n_node = Node {
          conn,
          g_score: total_g,
          f_score: total_g + Path::estimate(&conn, &to),
        };

        lookup.insert(conn, count);
        open.push(count);
        nodes.push(n_node);
        children.push(target);
        count += 1;
      }
    }

    None
  }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
struct Node {
  conn: Connection,
  g_score: i32,
  f_score: i32,
}
