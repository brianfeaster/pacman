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
use util::{Average, sleep, Term, neg_mod};
use gfx::*;

////////////////////////////////////////

fn square_diff(a: isize, b: usize) -> usize {
  let t = a - b as isize;
  (t*t) as usize
}

fn dist (x: isize, y: isize, mloc: &Mloc) -> usize {
  square_diff(x, mloc.x) + square_diff(y, mloc.y)
}

fn opposite (dir: usize) -> usize { & dir+2 & 3 }

fn nextdir (vid: &mut Graphics, loc: &Mloc, dir: usize, bias: usize) -> usize {
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
fn randir (vid: &mut Graphics, loc: &Mloc, dir: usize, bias: usize, wallIdx: usize) -> usize {
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
  let loc = &ghost.mlocField;
  let xg = loc.x();
  let yg = loc.y();
  let dir = ghost.dir;

  if WORLD.width <= xg || WORLD.height <= yg { return dir }

  let mlocFgoal = &ghost.mlocFgoal;

  let mut validDirs = vid
    .getFieldTiles(xg, yg, |t| (t as usize) < wallIdx)
    .map(|(hall, dir, xf, yf)| ( // Available directions scored with Pukman distance, otherwise MAX
       IF!(hall, dist(xf, yf, mlocFgoal), usize::MAX),
       dir));
  validDirs[opposite(dir)].0 = usize::MAX-1; // Opposite direction is MAX-1

  if ghost.isScared() { // "invert" scores so "away from Pukman" chosen
    validDirs
      .iter_mut()
      .for_each(|t| t.0 = usize::MAX/2 - t.0 )
  }

  validDirs.sort_by(|a,b|a.0.cmp(&b.0));
  validDirs[0].1
}

////////////////////////////////////////

trait Entity {
  fn enable (&mut self, vid :&mut Graphics);
  fn tick (&mut self);
}

// Amount to center sprite over its tile location
pub const SPRITECENTERX: usize = neg_mod((SPRITE.width-TILE.width)/2, SPRITEVIEW.width);
pub const SPRITECENTERY: usize = neg_mod((SPRITE.height-TILE.height)/2, SPRITEVIEW.height);

const DMX_SPRITEVIEW :[usize; 5] = [1, 0, neg_mod(1, SPRITEVIEW.width), 0, 0];
const DMY_SPRITEVIEW :[usize; 5] = [0, 1, 0, neg_mod(1, SPRITEVIEW.height), 0];

////////////////////////////////////////

struct Ghost {
    vid: Rc<RefCell<Graphics>>,
    pub tick: usize,
    sprite: usize,
    data: usize,
    dataScared: usize,
    dataEyes: usize,
    mlocField: Mloc, // Current field location
    dir: usize,      // direction of next location
    mlocFgoal: Mloc,
    state: usize, // 0 normal, _ ghost, MAX eyes
    score: usize
}

impl Ghost {
  fn new (vid: Rc<RefCell<Graphics>>,  sprite: usize, data: usize, dataScared:usize, dataEyes:usize, x:usize, y:usize, dir: usize) -> Ghost {
    Ghost{
      vid, tick:0, sprite, data, dataScared, dataEyes,
      mlocField:Mloc::new(SPRITEFIELD.width, SPRITEFIELD.height, x, y),
      dir, mlocFgoal:Mloc::new(SPRITEFIELD.width, SPRITEFIELD.height, 0,0),
      state:0, score:0
    }
  }
  fn setLocFgoal (&mut self, ml: &Mloc) { self.mlocFgoal.x=ml.x(); self.mlocFgoal.y=ml.y() }
  fn scared (&mut self) { if self.state != usize::MAX { self.state = 256 } }
  fn isScared (&self) -> bool { 0<self.state && usize::MAX!=self.state }
}

impl Entity for Ghost {
  fn enable (&mut self, vid :&mut Graphics) {
    vid.sprites[self.sprite].en = true;
  }
  fn tick (&mut self) {
    let mut wallIdx = 3;
    let isEyes = self.state==usize::MAX;

    let step = if isEyes {
      self.mlocFgoal.x=13; // Eyes' goal is home base
      self.mlocFgoal.y=14;
      wallIdx = 4;
      self.tick % 8
    } else {
      // Ghosts move at half rate / every other tick
      if 1 == self.tick&1 { self.tick+=1; return }
      (self.tick/2) % 8
    };

    // Get eaten or regenerate
    if self.mlocField.equal(&self.mlocFgoal) {
      if usize::MAX == self.state { self.state = 0 } // regenerate to ghost
      if 0 < self.state { self.score += 10000; self.state = usize::MAX }  // eaten to scared
    }

    // Get out of box
    if 11<=self.mlocField.x() && self.mlocField.x()<=16
        && 13<=self.mlocField.y() && self.mlocField.y()<=16 {
      self.mlocFgoal.x=13;
      self.mlocFgoal.y=12;
      wallIdx = 4
    }

    let vid = &mut self.vid.borrow_mut();

    let rx = (TILE.width*self.mlocField.x()  + step*DMX_SPRITEVIEW[self.dir] + SPRITECENTERX)%SPRITEVIEW.width;
    let ry = (TILE.height*self.mlocField.y() + step*DMY_SPRITEVIEW[self.dir] + SPRITECENTERY)%SPRITEVIEW.height;
    vid.setSpriteLoc(self.sprite, rx, ry);

    // Ghost animation
    vid.setSpriteIdx(self.sprite,
      match self.state {
        usize::MAX => self.dataEyes  +SPRITE.volume*(self.dir),
        0          => self.data      +SPRITE.volume*(self.dir*2 + self.tick/8%2),
        _          => self.dataScared+SPRITE.volume*(self.tick/8%2) } );


    if 7==step {
      self.mlocField.shift(self.dir);

      self.dir =
        if vid.rnd.rnd()&7 == 0 {
          randir(vid, &self.mlocField, self.dir, 4, wallIdx) // sometimes move randomly
        } else {
          dirGhostNew(vid, &self, wallIdx)
        };
    }

    self.tick += 1;
  } // fn tick
} // impl Entity for Ghost

////////////////////////////////////////

pub struct Pukman {
    vid: Rc<RefCell<Graphics>>,
    tick: usize,
    sprite: usize,
    data: usize,
    dir: usize,
    mlocField: Mloc,
    mlocRaster: Mloc,
    bias: usize,
    hungry: bool,
    score: usize
}

impl Pukman {
  fn new (vid: Rc<RefCell<Graphics>>, sprite: usize, data: usize, fx: usize, fy: usize, dir: usize) -> Pukman {
    Pukman{
      vid, tick:0, sprite, data, dir,
      mlocField:Mloc::new(SPRITEFIELD.width, SPRITEFIELD.height, fx, fy),
      mlocRaster: Mloc::new(SPRITEVIEW.width, SPRITEVIEW.height, 0, 0),
      bias:4, hungry:false, score:0 }
  }

  fn go (&mut self, go: usize) {
    if self.dir!=4 && go == opposite(self.dir) {
      // reverse direction by inverting the invariant: current cell becomes target, reverse step count
      let step = self.tick % 8;
      if 0 < step {
        self.mlocField.shift(self.dir);
        self.tick = 8 - step;
      }
      self.dir = go;
      self.bias = 4 // reversing direction removes move bias so movement can be random
    } else {
      self.bias = go
    }
  }
}

impl Entity for Pukman {
  fn enable (&mut self, vid :&mut Graphics) {
    vid.sprites[self.sprite].en = true;
  }
  fn tick (&mut self) {
    let step = self.tick % 8;

    let vid = &mut self.vid.borrow_mut();

    // resume movement
    if 4==self.dir && 4!=self.bias { self.dir = nextdir(vid, &self.mlocField, self.dir, self.bias) }

    // Maybe eat pill and become ghost-hungry if powerpill
    let x = self.mlocField.x();
    let y = self.mlocField.y();
    self.hungry = 0==step && x<WORLD.width && y<WORLD.height && {
      let dot = vid.setFieldTile(x, y, 0);
      self.score += match dot {
        1 => { 1 }
        2 => { 1000 }
        _ => 0
      };
      2 == dot
    };

    // tile-to-raster coordinates, center sprite on cell, parametric increment
    let rx = (TILE.width*x  + step*DMX_SPRITEVIEW[self.dir] + SPRITECENTERX)%SPRITEVIEW.width;
    let ry = (TILE.height*y + step*DMY_SPRITEVIEW[self.dir] + SPRITECENTERY)%SPRITEVIEW.height;
    self.mlocRaster.set(rx, ry);

    vid.setSpriteLoc(self.sprite, self.mlocRaster.x, self.mlocRaster.y);

//vid.msg = format!("\r\n{} {}  \r\n{} {}  \r\n", x, y, rx, ry);

    // Update animation
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


    if 7==step { // on next call, sprite will be at this target loc, new valid diretion, 0==tick%8
       self.mlocField.shift(self.dir);
       if 4!=self.dir || 4!=self.bias { self.dir = nextdir(vid, &self.mlocField, self.dir, self.bias) }
    }

    self.tick += 1;
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
  db: usize,
  esc: usize
}

impl ArcadeGame {
  fn new (mut vid: Graphics) -> Self {
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
    let db=offsets[7];
    vid.sprites[15].data = db;
    vid.sprites[15].mrloc = Mloc::new(VIEW.width, VIEW.height, 13*TILE.width, 15*TILE.width-TILE.width/2);
    vid.sprites[15].en = false;

    // Contain Graphics obj
    let vid : Rc<RefCell<Graphics>> = Rc::new(RefCell::new(vid));

    // Ghosts
    let ghosts = [
      Ghost::new(vid.clone(), 1, offsets[0], offsets[4], offsets[5], 13, 12, 2),  //blinky
      Ghost::new(vid.clone(), 2, offsets[1], offsets[4], offsets[5], 14, 15, 3),  //pinky
      Ghost::new(vid.clone(), 3, offsets[2], offsets[4], offsets[5], 13, 15, 2),  //inky
      Ghost::new(vid.clone(), 4, offsets[3], offsets[4], offsets[5], 15, 15, 0),  //clyde
    ];
    // Pukman
    let pukman = Pukman::new(vid.clone(), 0, offsets[6], 13, 18, 2);  // 13.5/26   5/17

    ArcadeGame{vid, keyboard, pukman, ghosts, digitsMem, dataTiles, db, esc:0}
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
      let Loc{x:left, y:top} = vid.topleft;
      let w = vid.viewportSize.x;
      
      vid.setSpriteLoc(5,left+25, top);
      vid.sprites[5].data = self.digitsMem+SPRITE.volume*(10+val%10);

      vid.setSpriteLoc(6,left+20, top);
      vid.sprites[6].data = self.digitsMem+SPRITE.volume*(10+val/10%10);

      vid.setSpriteLoc(7,left+15, top);
      vid.sprites[7].data = self.digitsMem+SPRITE.volume*(10+val/100%10);

      vid.setSpriteLoc(8,left+10, top);
      vid.sprites[8].data = self.digitsMem+SPRITE.volume*(10+val/1000%10);

      vid.setSpriteLoc(9,left+5, top);
      vid.sprites[9].data = self.digitsMem+SPRITE.volume*(10+val/10000%10);


      vid.setSpriteLoc(10,left+w-10, top);
      vid.sprites[10].data = self.digitsMem+SPRITE.volume*(10+score/1%10);

      vid.setSpriteLoc(11,left+w-15, top);
      vid.sprites[11].data = self.digitsMem+SPRITE.volume*(10+score/10%10);

      vid.setSpriteLoc(12,left+w-20, top);
      vid.sprites[12].data = self.digitsMem+SPRITE.volume*(10+score/100%10);

      vid.setSpriteLoc(13,left+w-25, top);
      vid.sprites[13].data = self.digitsMem+SPRITE.volume*(10+score/1000%10);

      vid.setSpriteLoc(14,left+w-30, top);
      vid.sprites[14].data = self.digitsMem+SPRITE.volume*(10+score/10000%10);
  }

  fn checkKeyboard (&mut self) -> bool {
    self.keyboard
      .try_recv()
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

  fn start(&mut self) {
    let mut mark = SystemTime::now();
    let mut avg = Average::new(256);
    let mut fps = 0;
    print!("\x1bc\x1b[?25l\x1b[0;37;40m");
    while self.checkKeyboard() {

      self.pukman.tick();
      self.ghosts.iter_mut().for_each(|g| {
        if self.pukman.hungry { g.scared() }
        g.setLocFgoal(&self.pukman.mlocField);
        g.tick()
      });

      let vid = &mut self.vid.borrow_mut();

      // Drinkybird
      vid.sprites[15].data = self.db + self.pukman.tick/8%2*256;

      let tr = self.pukman.mlocRaster;
      vid.alignViewport(tr.x(), tr.y()); // modulo SPRITEVIEWWIDTH = (28+2)*8 

      self.setFps(vid, fps, self.pukman.score + self.ghosts.iter().map(|g|g.score).sum::<usize>());
      vid.rasterizeTilesSprites(self.dataTiles);
//vid.msg = format!("{:?}", self.pukman.mlocRaster);
      vid.printField();

      // Next average frame time not include sleep time
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
