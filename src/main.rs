#![allow(non_snake_case)]

use std::{
  io::{stdin, Read},
  sync::mpsc::{channel, Receiver},
  time::{SystemTime},
  thread,
};

mod data;
mod util;
mod gfx;

use data::{initializeVideoDataPukman};
use util::{sleep, Term};
use gfx::*;

////////////////////////////////////////

const fn neg_mod (n: usize, b: usize) -> usize { b-n }
const SPRITECENTERX: usize = neg_mod((SPRITEWIDTH-TILEWIDTH)/2, SPRITERASTERWIDTH);
const SPRITECENTERY: usize = neg_mod((SPRITEHEIGHT-TILEHEIGHT)/2, SPRITERASTERHEIGHT);

////////////////////////////////////////

fn square_diff (a: isize, b: usize) -> usize { let t = a - b as isize; (t*t) as usize }
fn dist (x: isize, y: isize, loc: Loc) -> usize { square_diff(x, loc.x) + square_diff(y, loc.y) }
fn opposite (dir: usize) -> usize { [3, 2, 1, 0, 4][dir] }

fn randir (vid: &mut Graphics, Loc{x,y}: Loc, dir: usize, bias: usize) -> usize {
  if FIELDWIDTH <= x || FIELDHEIGHT <= y { return dir } // if off field, keep going / wrap around
  let mut validDirs = vid.getFieldTiles(x, y, |t|IF!(t<3, 7, 1));
  validDirs[opposite(dir)].0 -= 3;
  if 4!=bias {
    validDirs[bias].0 += 1;
    validDirs[opposite(bias)].0 -= 1;
  }
  validDirs.sort();
  let r =
    IF!(validDirs[3].0 == validDirs[2].0,
      IF!(validDirs[2].0 == validDirs[1].0,
        3,
        2),
      1);
  validDirs[3-vid.rnd.rnd() as usize % r].1
}

fn dirNew (vid: &mut Graphics, Loc{x,y}: Loc, dir: usize, locP:Loc, scared: bool) -> usize {
  if FIELDWIDTH <= x || FIELDHEIGHT <= y { return dir }
  let mut validDirs = vid
    .getFieldTiles(x, y, |t| t<3)
    .map(|(hall, dir, xp, yp)| (
       IF!(hall, dist(xp, yp, locP), usize::MAX),
       dir));
  validDirs[opposite(dir)].0 = usize::MAX-1;
  if scared { validDirs.iter_mut().for_each(|t| t.0 = usize::MAX/2 - t.0 ) }
  validDirs.sort_by(|a,b|a.0.cmp(&b.0));
  validDirs[0].1
}

////////////////////////////////////////

trait Entity {
  fn enable (&mut self, vid :&mut Graphics);
  fn tick (&mut self, vid: &mut Graphics);
}

const DMX_SPRITEFIELD :[usize; 4] = [0, neg_mod(1, SPRITEFIELDWIDTH), 1, 0];
const DMY_SPRITEFIELD :[usize; 4] = [neg_mod(1, SPRITEFIELDHEIGHT), 0, 0, 1];

const DMX_TILE :[usize; 4] = [0, neg_mod(1, SPRITERASTERWIDTH), 1, 0];
const DMY_TILE :[usize; 4] = [neg_mod(1, SPRITERASTERHEIGHT), 0, 0, 1];

////////////////////////////////////////

struct Ghost {
    tick: usize,
    sprite: usize,
    data: usize,
    dataScared: usize,
    dataEyes: usize,
    locField: Loc,
    dir: usize,
    locDesired: Loc,
    state: usize, // 0 normal, _ ghost, MAX eyes
}

impl Ghost {
  fn new (sprite: usize, data: usize, dataScared:usize, dataEyes:usize, x:usize, y:usize, dir: usize) -> Ghost {
    Ghost{tick:0, sprite, data, dataScared, dataEyes, locField:Loc{x,y}, dir, locDesired:Loc::default(), state:0}
  }
  fn setDesiredLoc (&mut self, loc: Loc) { self.locDesired = loc }
  fn scared (&mut self) { if self.state != usize::MAX { self.state = 256 } }
}

impl Entity for Ghost {
  fn enable (&mut self, vid :&mut Graphics) {
    vid.sprites[self.sprite].en = true;
  }
  fn tick (&mut self, vid: &mut Graphics) {
    if 1 == self.tick&1 { self.tick+=1; return }
    let inc = (self.tick/2) % 8;

    // When eyes reset desired loc to base
    if usize::MAX == self.state { self.locDesired = Loc{x:13, y:14}; }

    // Get eaten or regenerate
    if self.locField == self.locDesired {
      if usize::MAX == self.state { self.state = 0 } // regenerate to ghost
      if 0 < self.state { self.state = usize::MAX }  // eaten to scared
    }

    vid.setSpriteLoc(self.sprite, Loc{
      x: (TILEWIDTH*self.locField.x  + inc*DMX_TILE[self.dir] + SPRITECENTERX) % SPRITERASTERWIDTH,
      y: (TILEHEIGHT*self.locField.y + inc*DMY_TILE[self.dir] + SPRITECENTERY) % SPRITERASTERHEIGHT
    });

    // Ghost animation
    match self.state {
      usize::MAX => vid.setSpriteIdx(self.sprite, self.dataEyes  +SPRITEVOLUME*(self.dir)),
      0          => vid.setSpriteIdx(self.sprite, self.data      +SPRITEVOLUME*(self.dir*2 + self.tick/8%2)),
      _          => vid.setSpriteIdx(self.sprite, self.dataScared+SPRITEVOLUME*(self.tick/8%2))
    }

    if 7 == inc {
      self.locField.x = (self.locField.x + DMX_SPRITEFIELD[self.dir]) % SPRITEFIELDWIDTH;
      self.locField.y = (self.locField.y + DMY_SPRITEFIELD[self.dir]) % SPRITEFIELDHEIGHT;

      self.dir =
        if false && vid.rnd.rnd()&7 == 0 {
          randir(vid, self.locField, self.dir, 4) // sometimes move randomly
        } else {
          dirNew(vid, self.locField, self.dir, self.locDesired, 0<self.state && usize::MAX!=self.state)
        };
    }
    self.tick += 1;
  } // fn tick
} // impl Entity for Ghost

////////////////////////////////////////

pub struct Pukman {
    tick: usize,
    sprite: usize,
    data: usize,
    dir: usize,
    locField: Loc, locRaster: Loc,
    go: usize,
    bias: usize,
    hungry: bool
}

impl Pukman {
  fn new (sprite: usize, data: usize, fx: usize, fy: usize, dir: usize) -> Pukman {
    Pukman{tick:0, sprite, data, dir, locField:Loc{x:fx, y:fy}, locRaster: Loc::default(), go:4, bias:4, hungry:false}
  }
  fn go (&mut self, dir: usize) { self.go = dir }
}

impl Entity for Pukman {
  fn enable (&mut self, vid :&mut Graphics) {
    vid.sprites[self.sprite].en = true;
  }
  fn tick (&mut self, vid: &mut Graphics) {
    let mut inc = self.tick % 8;

    // EatPill
    self.hungry =
      0 == inc
      && self.locField.x < FIELDWIDTH
      && self.locField.y < FIELDHEIGHT
      && 2 == vid.setFieldTile(self.locField, 0);

    // reverse direction
    if self.go == opposite(self.dir) {
      if 0 < inc {
        self.locField.x = (self.locField.x + DMX_SPRITEFIELD[self.dir]) % SPRITEFIELDWIDTH;
        self.locField.y = (self.locField.y + DMY_SPRITEFIELD[self.dir]) % SPRITEFIELDHEIGHT;
        self.tick = 8 - inc;
        inc = self.tick % 8;
      }
      self.dir = self.go;
    }
    if 4 != self.go {
      self.bias = self.go;
      self.go = 4;
    }

    // tile-to-raster coordinates, center sprite on cell, parametric increment
    self.locRaster.x = (TILEWIDTH*self.locField.x  + inc*DMX_TILE[self.dir] + SPRITECENTERX) % SPRITERASTERWIDTH;
    self.locRaster.y = (TILEHEIGHT*self.locField.y + inc*DMY_TILE[self.dir] + SPRITECENTERY) % SPRITERASTERHEIGHT;
    vid.setSpriteLoc(self.sprite, self.locRaster);

    if 7 == inc { // on next call, sprite will be at this target loc, new valid diretion, 0==tick%8
       self.locField.x = (self.locField.x + DMX_SPRITEFIELD[self.dir]) % SPRITEFIELDWIDTH;
       self.locField.y = (self.locField.y + DMY_SPRITEFIELD[self.dir]) % SPRITEFIELDHEIGHT;
       self.dir = randir(vid, self.locField, self.dir, self.bias);
    }

    // Pukman animation
    vid.setSpriteIdx(self.sprite,
      self.data + match self.tick%4 {
        0 => 0,
        1 => self.dir * 2 + 1,
        2 => self.dir * 2 + 2,
        3 => self.dir * 2 + 1,
        _ => 0
      } * SPRITEVOLUME
    );

    self.tick += 1;
  }
} // impl Entity for Pukman

////////////////////////////////////////

struct ArcadeGame {
  vid: Graphics,
  keyboard: Receiver<u8>,
  pukman: Pukman,
  ghosts: [Ghost; 4],
  digitsMem: usize,
  dataTiles: usize
}

impl ArcadeGame {

  fn new (mut vid: Graphics) -> Self {
    let keyboard = ArcadeGame::initKeyboardReader();

    let offsets = initializeVideoDataPukman(&mut vid);
    let mut ghosts = [
      Ghost::new(1, offsets[0], offsets[4], offsets[5],  3, 17, 1),  //blinky
      Ghost::new(2, offsets[1], offsets[4], offsets[5], 18, 20, 1),  //pinky
      Ghost::new(3, offsets[2], offsets[4], offsets[5],  9, 14, 2),  //inky
      Ghost::new(4, offsets[3], offsets[4], offsets[5], 18, 14, 3),
    ]; //clyde
    let mut pukman = Pukman::new(0, offsets[6], 6, 17, 1);  // 13.5 26
    let digitsMem = offsets[7];
    let dataTiles = offsets[8];

    // Enable sprites:  pukman, ghosts, FPS digits
    ghosts.iter_mut().for_each(|g| g.enable(&mut vid));
    pukman.enable(&mut vid);
    vid.sprites[5].en = true;
    vid.sprites[6].en = true;
    vid.sprites[7].en = true;
    vid.sprites[8].en = true;
    vid.sprites[9].en = true;

    ArcadeGame{vid, keyboard, pukman, ghosts, digitsMem, dataTiles}
  } // fn new

  fn initKeyboardReader () -> Receiver<u8> {
    let (fifo_in, fifo_out) = channel::<u8>();
    let mut si = stdin();
    thread::spawn(move || loop {
      let mut b = [0];
      si.read(&mut b).unwrap();
      fifo_in.send(b[0]).unwrap();
    });
    fifo_out
  }

  fn setScore (&mut self, val: usize) {
      let Loc{x:left, y:top} = self.vid.topleft;
      self.vid.sprites[5].loc.x = left+32;
      self.vid.sprites[5].loc.y = top+1;
      self.vid.sprites[5].data=self.digitsMem+SPRITEVOLUME*(val%10);

      self.vid.sprites[6].loc.x = left+24;
      self.vid.sprites[6].loc.y = top+1;
      self.vid.sprites[6].data=self.digitsMem+SPRITEVOLUME*(val/10%10);

      self.vid.sprites[7].loc.x = left+16;
      self.vid.sprites[7].loc.y = top+1;
      self.vid.sprites[7].data=self.digitsMem+SPRITEVOLUME*(val/100%10);

      self.vid.sprites[8].loc.x = left+8;
      self.vid.sprites[8].loc.y = top+1;
      self.vid.sprites[8].data=self.digitsMem+SPRITEVOLUME*(val/1000%10);

      self.vid.sprites[9].loc.x = left;
      self.vid.sprites[9].loc.y = top+1;
      self.vid.sprites[9].data=self.digitsMem+SPRITEVOLUME*(val/10000%10);
  }

  fn start(&mut self) {
    let mut mark = SystemTime::now();
    let mut dur = 0;
    const FPS_SAMPLES: usize = 50;
    let mut fps: [usize; FPS_SAMPLES] = [0; FPS_SAMPLES];
    let mut fpsCount=0;
    let mut fpsp=0;
    let mut sum=0;
    print!("\x1bc\x1b[0;37;40m\x1b[H\x1b[J");
    loop {

      match self.keyboard.try_recv() {
        Ok(3)|Ok(27)|Ok(81)|Ok(113) => break, // Quit: ^C ESC q Q
        Ok(104)|Ok(97)  => self.pukman.go(1), // Left: h a
        Ok(108)|Ok(100) => self.pukman.go(2), // Right: l d
        Ok(107)|Ok(119) => self.pukman.go(0), // Up: k w
        Ok(106)|Ok(115) => self.pukman.go(3), // Down: j s
        _ => ()
      }

      self.pukman.tick(&mut self.vid);

      self.ghosts.iter_mut().for_each(|g| {
        if self.pukman.hungry { g.scared() }
        g.setDesiredLoc(self.pukman.locField);
        g.tick(&mut self.vid)
      });

      self.setScore(dur);

      self.vid.rasterizeTilesSprites(self.dataTiles);
      self.vid.printField(self.pukman.locRaster);

      dur = mark.elapsed().unwrap().as_micros() as usize;
      sum = sum - fps[fpsp] + dur;
      fps[fpsp] = dur;
      fpsp = (fpsp+1)%FPS_SAMPLES;
      if fpsCount != FPS_SAMPLES { fpsCount += 1 }
      dur = 1000000/(sum / fpsCount) as usize;
      if 99999 < dur { dur = 99999; }

      sleep(30);

      mark = SystemTime::now();
    }
    print!("\x1b[m");
  } // fn start

} // impl ArcadeGame

////////////////////////////////////////

fn main() {
  ArcadeGame::new(Graphics::new(Term::new())).start();
}
