use rand::{Rng, thread_rng};

use ggez::{
  Context,
  graphics::{
    self,
    Point2,
    Color,
  },
  GameResult,
  timer::{
    get_delta,
    duration_to_f64,
  },
};

use path::track::{
  Track,
  TrackPiece,
};

pub struct Train {
  segments: Vec<Segment>,
  colour: Color,
}

impl Train {
  pub fn new(speed: f32, track: usize, dist: f32, (seg_n, seg_dist, seg_len): (usize, f32, f32)) -> Self {
    let mut rnd = thread_rng();

    let colour: Color = [rnd.gen_range(0.0, 1.0), rnd.gen_range(0.0, 1.0), rnd.gen_range(0.0, 1.0), 1.0].into();

    let mut segments = Vec::new();

    let mut last = dist;

    for _ in 0..seg_n {
      segments.push(Segment::new(speed, track, last));
      segments.push(Segment::new(speed, track, last + seg_len));

      last += seg_len + seg_dist;
    }

    Train {
      segments,
      colour,
    }
  }

  pub fn update(&mut self, ctx: &mut Context, tracks: &Vec<Track>) {
    let delta = duration_to_f64(get_delta(ctx)) as f32;


    for seg in self.segments.iter_mut() {
      seg.update(tracks, delta);
    }
  }

  pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
    graphics::set_color(ctx, self.colour)?;

    for seg in self.segments.iter_mut() {
      seg.draw(ctx)?;
    }

    for conn in self.segments.chunks(2) {
      let (start, end) = (conn[0].pos, conn[1].pos);
      let (start_p, end_p) = (Point2::new(start.0, start.1), Point2::new(end.0, end.1));
      graphics::line(ctx, &[start_p, end_p], 10.)?;
    }

    Ok(())
  }
}

pub struct Segment {
  speed: f32,
  track: usize,
  dist: f32,
  pos: (f32, f32),
}

impl Segment {
  pub fn new(speed: f32, track: usize, dist: f32) -> Self {
    Segment {
      speed,
      track,
      dist,
      pos: (0., 0.),
    }
  }

  pub fn update(&mut self, tracks: &Vec<Track>, delta: f32) {
    let mut track = tracks.get(self.track).expect("tracks should have the current one");
    let mut len = track.len();

    self.dist += self.speed * delta;

    while self.dist > len || self.dist < 0. {
      if self.dist > len {
        self.dist = self.dist - len;
        if let Some(trc) = tracks.get(self.track + 1) {
          track = trc;
          len = track.len();
          self.track += 1;
        } else {
          self.dist = len - self.dist;
          self.speed = -self.speed;
        }
      }

      if self.dist < 0. {
        self.dist = -self.dist;
        if let Some(trc) = tracks.get(self.track - 1) {
          track = trc;
          len = track.len();
          self.track -= 1;
          self.dist = len as f32 - self.dist;
        } else {
          self.speed = -self.speed;
        }
      }
    }

    let perc = self.dist / len as f32;
    let pos = track.lerp(perc);

    self.pos = pos.to_float();
  }

  pub fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
    graphics::circle(ctx, graphics::DrawMode::Fill, Point2::new(self.pos.0, self.pos.1), 5., 0.2)?;

    Ok(())
  }
}
