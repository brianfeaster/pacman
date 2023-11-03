#![allow(non_snake_case)]

use std::{
  cell::RefCell,
  io::{stdin, Read},
  rc::Rc,
  sync::mpsc::{channel, Receiver},
  time::{SystemTime},
  thread,
};

mod data;
mod util;
mod gfx;

use data::{initializeVideoDataPukman};
use util::{Average, sleep, Term};
use gfx::*;

////////////////////////////////////////

fn square_diff(a: isize, b: usize) -> usize {
  let t = a - b as isize;
  (t*t) as usize
}

fn dist (x: isize, y: isize, mloc: &Mvec) -> usize {
  square_diff(x, mloc.x) + square_diff(y, mloc.y)
}

fn opposite (dir: usize) -> usize { match dir { 4 => 4, dir => dir+2 & 3 } }

fn nextdir (vid: &mut Graphics, loc: &Mvec, dir: usize, bias: usize) -> usize {
  let validDirs = vid.getFieldTiles(loc.x(), loc.y(), |t|t<3);

  if dir==4 {
    // maybe resume movement
    if bias != 4 && validDirs[bias].0 { bias } else { 4 }
  } else {
    if bias!=4 && validDirs[bias].0 {
      // change direction while moving
      bias
    } else {
      // continue moving or stop
      if validDirs[dir].0 { dir } else { 4 }
    }
  }
}

fn randir (vid: &mut Graphics, loc: &Mvec, dir: usize, bias: usize, wallIdx: usize) -> usize {
  let x = loc.x();
  let y = loc.y();
  if WORLD.width <= x || WORLD.height <= y { return dir } // If off field dont change direction.
  // Weighted scoring to determine next direction.
  let mut validDirs = vid.getFieldTiles(x, y, |t|IF!((t as usize) < wallIdx, 7, 1)); // Available direction score: +7, else +1 so no final score negative
  validDirs[opposite(dir)].0 -= 3; // Reverse current direction: -3, only go backwards if dead-end
  if 4!=bias {
    validDirs[bias].0 += 1;        // Desired direction: +1
    validDirs[opposite(bias)].0 -= 1; // Opposite desired direction: -1 (IE don't go down if user wants to go up eventually)
  }
  validDirs.sort();
  let r = // Tied best scored chosen randomly
    IF!(validDirs[3].0 == validDirs[2].0,
      IF!(validDirs[2].0 == validDirs[1].0,
        3,
        2),
      1);
  validDirs[3-vid.rnd.rnd() as usize % r].1
}

fn dirGhostNew (vid: &mut Graphics, ghost: &Ghost, wallIdx: usize) -> usize {
  let loc = &ghost.locField; // modulo spriteField
  let xg = loc.x();
  let yg = loc.y();
  let dir = ghost.dir;

  let mut validDirs = vid
    .getFieldTiles(xg, yg, |t| (t as usize) < wallIdx)
    .map(|(hall, dir, xf, yf)| ( // Available directions scored with Pukman distance, otherwise MAX
       IF!(hall, dist(xf, yf, &ghost.goalSpritefieldMvec), usize::MAX),
       dir));
  validDirs[opposite(dir)].0 = usize::MAX-1; // Opposite direction is MAX-1

  if ghost.isScared() && wallIdx==3 { // "invert" scores so "away from Pukman" chosen unless in box
    validDirs
      .iter_mut()
      .for_each(|t| t.0 = usize::MAX/2 - t.0 )
  }

  validDirs.sort_by(|a,b|a.0.cmp(&b.0));
  let r = // Tied best scored chosen randomly
    IF!(validDirs[0].0 == validDirs[1].0,
      IF!(validDirs[1].0 == validDirs[2].0,
        3,
        2),
      1);
  validDirs[vid.rnd.rnd() as usize % r].1
}

////////////////////////////////////////

trait Entity {
  fn render (&mut self);
  fn tick (&mut self);
}

////////////////////////////////////////

struct Ghost {
    vid: Rc<RefCell<Graphics>>,
    pub tick: usize,
    sprite: usize,
    data: usize,
    dataScared: usize,
    dataEyes: usize,
    dir: usize,      // direction of next location
    locField: Mvec, // Current field location
    goalSpritefieldMvec: Mvec,
    state: usize, // 0 normal, _ ghost, MAX eyes
    score: usize
}

impl Ghost {
  fn new (
    vid: Rc<RefCell<Graphics>>,
    sprite: usize,
    data: usize,
    dataScared:usize,
    dataEyes:usize,
    x:usize, y:usize,
    dir: usize
  ) -> Ghost {
    let (locField, goalSpritefieldMvec) = {
      let mut v = vid.borrow_mut();
      let loc = (
        Mvec::new(v.spriteField.w, v.spriteField.h, x, y),
        Mvec::new(v.spriteField.w, v.spriteField.h, x, y)
      );
      v.sprites[sprite].locWindow.w = v.spriteView.w;
      v.sprites[sprite].locWindow.h = v.spriteView.h;
      loc
    };
    //if 4!=dir { locField.shift(opposite(dir), 1); }
    let mut g = Ghost{
      vid, tick:0,
      sprite, data, dataScared, dataEyes, dir,
      locField, goalSpritefieldMvec,
      state:0, score:0
    };
    g.render();
    g
  }
  fn setLocFgoal (&mut self, ml: &Mvec) { self.goalSpritefieldMvec.x=ml.x(); self.goalSpritefieldMvec.y=ml.y() }
  fn scared (&mut self) { if self.state != usize::MAX { self.state = 256 } }
  fn isScared (&self) -> bool { 0<self.state && usize::MAX!=self.state }
}

impl Entity for Ghost {

  fn render (&mut self) {
    let vid = &mut self.vid.borrow_mut();
    // location
    let (x, y) = (
      (self.locField.x() * TILE.width  + vid.spriteTileCenterAdj.x)%vid.spriteView.w,
      (self.locField.y() * TILE.height + vid.spriteTileCenterAdj.y)%vid.spriteView.h
    );
    vid.setSpriteLocWindow(self.sprite, x, y);
    // step
    vid.shiftSprite(self.sprite, self.dir, self.tick/2 % 8);
    // Ghost animation
    vid.setSpriteIdx(self.sprite,
      match self.state {
        usize::MAX => self.dataEyes  +SPRITE.volume*(self.dir),
        0          => self.data      +SPRITE.volume*(self.dir*2 + self.tick/2/8%2),
        _          => self.dataScared+SPRITE.volume*(self.tick/2/8%2)
      });
  }

  fn tick (&mut self) {
    let mut wallIdx = 3;

    if 1<(self.state+1) && self.locField.equal(&self.goalSpritefieldMvec) {
      // ghost eaten, becomes eyes
      self.state = usize::MAX;
      self.score += 10000;
    }

    if self.state == usize::MAX {
      // Eyes' goal is home base
      self.goalSpritefieldMvec.x=14;
      self.goalSpritefieldMvec.y=15;
      wallIdx = 4; // can walk through door
    }

    self.tick += 1;

    // Ghosts move at half rate, eyes don't
    if 1 == (self.tick&1) {
      if self.state == usize::MAX { self.tick +=1 }
      else { return }
    }

    if 0 == (self.tick/2) % 8 {
      self.locField.shift(self.dir, 1);
    }

    if self.state == usize::MAX && self.locField.equal(&self.goalSpritefieldMvec) {
      // eyes regenerate to ghost in base
      self.state = 0;
    }

    // Ghosts want to leave box (coordinates includes box border)
    if self.state != usize::MAX
        && 10<=self.locField.x() && self.locField.x()<=17
        && 13<=self.locField.y() && self.locField.y()<=17 {
      self.goalSpritefieldMvec.x=13;
      self.goalSpritefieldMvec.y=0;
      self.state = 0;
      wallIdx = 4
    }

    if 0 == (self.tick/2) % 8 {
      let vid = &mut self.vid.borrow_mut();
      self.dir =
        if self.state==0 && vid.rnd.rnd()&7 == 0 {
          randir(vid, &self.locField, self.dir, 4, wallIdx) // sometimes move randomly
        } else {
          dirGhostNew(vid, &self, wallIdx)
        };
    }

    self.render();
  } // fn tick
} // impl Entity for Ghost

////////////////////////////////////////

pub struct Pukman {
    vid: Rc<RefCell<Graphics>>,
    tick: usize,
    sprite: usize,
    data: usize,
    locField: Mvec,
    dir: usize,
    bias: usize,
    hungry: bool,
    score: usize
}

impl Pukman {
  fn new (
    vid: Rc<RefCell<Graphics>>,
    sprite: usize,
    data: usize,
    x: usize,
    y: usize,
    dir: usize
  ) -> Pukman {
    let locField = {
      let mut v = vid.borrow_mut();
      let mut loc = Mvec::new(v.spriteField.w, v.spriteField.h, x, y);
      if 4!=dir { loc.shift(opposite(dir), 1); }
      v.sprites[sprite].locWindow.w = v.spriteView.w;
      v.sprites[sprite].locWindow.h = v.spriteView.h;
      loc
    };
    let mut p = Pukman{
      vid, tick: match dir { 4=>0, _=>usize::MAX },
      sprite, data,
      dir, locField, bias:4,
      hungry:false, score:0
    };
    if 4==dir { p.render() }
    p
  }
  // reverse direction by inverting the invariant: current cell becomes target, reverse step count
  fn reverse (&mut self) {
      let step = self.tick % 8;
      if 0 < step {
        self.locField.shift(self.dir, 1);
        self.tick = self.tick + 8 - 2*step;
      }
      self.dir = opposite(self.dir);
  }

  fn go (&mut self, go: usize) {
    self.bias = go
  }
}

impl Entity for Pukman {
  fn render (&mut self) {
    let vid = &mut self.vid.borrow_mut();

    // Update sprite's location
    let (x, y) = (
      ((self.locField.x()*TILE.width  + vid.spriteTileCenterAdj.x))%vid.spriteView.w,
      ((self.locField.y()*TILE.height + vid.spriteTileCenterAdj.y))%vid.spriteView.h
    );
    vid.setSpriteLocWindow(self.sprite, x, y);

    // Update sprite's step location
    vid.shiftSprite(self.sprite, self.dir, self.tick%8);

    // Update sprite's animation
    vid.setSpriteIdx(self.sprite, self.data + SPRITE.volume *
      match self.dir {
        4 => 0,
        _ => match self.tick%4 {
              1|3 => self.dir * 2 + 1,
              2   => self.dir * 2 + 2,
              _   => 0
             }
      }
    );
  } // fn render

  fn tick (&mut self) {
    if 4==self.dir {
      self.dir = nextdir(&mut self.vid.borrow_mut(), &self.locField, self.dir, self.bias);
      if 4==self.dir { return }
    } else if self.bias == opposite(self.dir) {
      self.reverse()
    }
    self.tick += 1;
    if 0 == self.tick%8 {
      let vid = &mut self.vid.borrow_mut();
      // Upddate field loc
      self.locField.shift(self.dir, 1);
      // Maybe eat pill and become ghost-hungry if powerpill
      let dot = vid.setFieldTile(self.locField.x(), self.locField.y(), 0);
      self.score += match dot {
        1 => { 1 }
        2 => { self.hungry=true; 1000 }
        _ => 0
      };
      self.dir = nextdir(vid, &self.locField, self.dir, self.bias);
    }
    self.render();
  } // fn tick

} // impl Entity for Pukman

////////////////////////////////////////////////////////////////////////////////

struct ArcadeGame {
  vid: Rc<RefCell<Graphics>>,
  keyboard: Receiver<u8>,
  pukman: Pukman,
  ghosts: [Ghost; 4],
  digitsMem: usize,
  dataTiles: usize,
  drinkyBirdData: usize,
  esc: usize
}

impl ArcadeGame {
  fn new (mut vid: Graphics) -> Self {
    vid.setFieldSize(28, 33);
    let keyboard = ArcadeGame::initKeyboardReader();

    // Load sprite and tile data
    let offsets = initializeVideoDataPukman(&mut vid);

    // Digits
    let digitsMem = offsets[8];

    // Tiles
    let dataTiles = offsets[9];

    // Enable all sprites
    (0..16).for_each(|i| vid.sprites[i].en = true);

    // Drinkybird
    let drinkyBirdData = offsets[7];
    vid.sprites[15].data = drinkyBirdData;
    vid.setSpriteLocWindow(15, 13*TILE.width, 15*TILE.width-TILE.width/2);

    // Ghosts
    let vid : Rc<RefCell<Graphics>> = Rc::new(RefCell::new(vid));
    let ghosts = [
      Ghost::new(vid.clone(), 1, offsets[0], offsets[4], offsets[5],  9, 12, 0),  //blinky
      Ghost::new(vid.clone(), 2, offsets[1], offsets[4], offsets[5], 18, 12, 1),  //pinky
      Ghost::new(vid.clone(), 3, offsets[2], offsets[4], offsets[5],  9, 18, 3),  //inky
      Ghost::new(vid.clone(), 4, offsets[3], offsets[4], offsets[5], 18, 18, 2),  //clyde
    ];
    // Pukman
    let pukman = Pukman::new(vid.clone(), 0, offsets[6], 15, 18, 2);  // 13.5/24

    ArcadeGame{vid, keyboard, pukman, ghosts, digitsMem, dataTiles, drinkyBirdData, esc:0}
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

  fn setFps (&self, vid: &mut Graphics, val: usize, score: usize) {
      let Mvec{x:left, y:top, w:width, h:_} = vid.rasterView;
      (5..=9).into_iter().for_each(|i| {
        vid.setSpriteLocWindow(i,left+25-(i-5)*5, top);
        vid.sprites[i].data = self.digitsMem+SPRITE.volume*(10+val/10_usize.pow(i as u32 -5)%10);
      });

      (10..=14).into_iter().for_each(|i| {
        vid.setSpriteLocWindow(i,left+width-(i-8)*5, top);
        vid.sprites[i].data = self.digitsMem+SPRITE.volume*(10+score/10_usize.pow(i as u32 -10)%10);
      });
  }

  fn checkKeyboard (&mut self) -> bool {
    self.keyboard
      .try_recv()
      //.recv_timeout(Duration::from_millis(60*1000))
      .map(|k| {
        match k as char {
        '\x03'|'q'|'Q' => return false, // Quit: ^C q Q
        '\x1b' => { self.esc += 1; return self.esc != 2 }
        'l'|'d'|'C' => self.pukman.go(0), // Right: l d
        'j'|'s'|'B' => self.pukman.go(1), // Down: j s
        'h'|'a'|'D' => self.pukman.go(2), // Left: h a
        'k'|'w'|'A' => self.pukman.go(3), // Up: k w
        _ => ()
        }
        self.esc = 0;
        return true
      })
      .unwrap_or(true)
  }

  fn renderFrame(&mut self, fps: usize, score: usize) {
    let vid = &mut self.vid.borrow_mut();
    vid.centerRasterView(self.pukman.sprite);
    self.setFps(vid, fps, score);
    vid.rasterizeTilesSprites(self.dataTiles);
    vid.printField();
  }

  fn start(&mut self) {
    let mut mark = SystemTime::now();
    let mut avg = Average::new(256);
    let mut fps = 0;
    print!("\x1bc\x1b[?25l\x1b[0;37;40m");

    while self.checkKeyboard() {
      self.pukman.tick();
      self.ghosts.iter_mut().for_each(|g| {
        if self.pukman.hungry { g.scared() }
        g.setLocFgoal(&self.pukman.locField);
        g.tick()
      });
      self.pukman.hungry=false;
      self.vid.borrow_mut().sprites[15].data = self.drinkyBirdData + self.pukman.tick/8%2*256; // Drinkybird
      self.renderFrame(fps, self.pukman.score + self.ghosts.iter().map(|g|g.score).sum::<usize>());
      // Nex5 average frame time not include sleep time
      fps = 1000000 / avg.add(mark.elapsed().unwrap_or_default().as_micros() as usize);
      sleep(40);
      mark = SystemTime::now();
    }
    println!("\x1b[m\x1b[?25h");
  } // fn start
} // impl ArcadeGame

////////////////////////////////////////

fn main() {
  ArcadeGame::new(Graphics::new(Term::new())).start()
}
