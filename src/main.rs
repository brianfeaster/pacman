#![allow(non_snake_case)]

use std::{
  cmp::{max, min},
  collections::HashMap,
  fmt::Write,
  io::{self, stdout, Stdout, stdin, Read},
  sync::mpsc::{channel, Receiver},
  time::{SystemTime},
  thread,
  usize
};

mod data;
mod util;

use data::{initializeVideoDataPukman};
pub use util::{sleep, readline, Rnd, Term};

////////////////////////////////////////

// Pixels ᗣ ᗧ · ⋅ • ● ⬤
pub const BLKA :&str = "\x1b[3m "; // Inverted ASCII space
pub const BLKB :char = '█'; // full block 2588
pub const BLK  :char = '▉'; // left 7/8th block 2589
pub const BLKD :char = '◼'; // black medium square 25fc
pub const BLKE :char = '▮'; // black verticle rectangle 25ae
pub const BLKF :char = '▪'; // black small square 25aa

////////////////////////////////////////

const MEMORYSIZE: usize = 256*256; // 64k

const TILEWIDTH: usize  = 8;
const TILEHEIGHT: usize = 8;
const TILEVOLUME: usize = TILEWIDTH*TILEHEIGHT;

// tile field 28x36
const FIELDWIDTH: usize  = 28;
const FIELDHEIGHT: usize = 36;
const FIELDVOLUME: usize = FIELDWIDTH*FIELDHEIGHT;

const SPRITEWIDTH: usize = 16;
const SPRITEHEIGHT: usize= 16;
const SPRITEVOLUME: usize= SPRITEWIDTH*SPRITEHEIGHT;
const SPRITECOUNT: usize = 10;

// Includes non-visible right/bottom borders
const RASTERWIDTH:  usize = TILEWIDTH * FIELDWIDTH;
const RASTERHEIGHT: usize = TILEHEIGHT * FIELDHEIGHT;

const SPRITEFIELDWIDTH: usize  = FIELDWIDTH + SPRITEWIDTH/TILEWIDTH;
const SPRITEFIELDHEIGHT: usize = FIELDHEIGHT + SPRITEHEIGHT/TILEHEIGHT;

const SPRITERASTERWIDTH: usize  = SPRITEFIELDWIDTH * TILEWIDTH;
const SPRITERASTERHEIGHT: usize = SPRITEFIELDHEIGHT * TILEHEIGHT;

#[derive(Clone,Copy,Default)]
struct Loc {
  x: usize,
  y: usize,
}

#[derive(Clone,Copy,Default)]
struct Sprite {
    data: usize,
    loc: Loc,
    en: bool,
}

pub struct Graphics {
    term: Term,
    memory: [u8; MEMORYSIZE],
    memPtr: usize,
    c2b:    HashMap<char, u8>,
    field:   [u8; FIELDVOLUME],
    update:  [bool; FIELDVOLUME],
    sprites: [Sprite; SPRITECOUNT],
    raster: [u8; TILEVOLUME*FIELDVOLUME],
    last:   [u8; TILEVOLUME*FIELDVOLUME],
    rnd: Rnd,
    msg: String,
    topleft: Loc
}

impl Graphics {
  fn new(term: Term) -> Graphics {
    Graphics{
      term,
      memory:  [0; MEMORYSIZE],
      memPtr:  0,
      c2b:     HashMap::<char, u8>::new(),
      field:     [63; FIELDVOLUME],
      update:    [true; FIELDVOLUME],
      sprites:   [Sprite::default(); SPRITECOUNT],
      raster:   [0;  (TILEVOLUME*FIELDVOLUME)],
      last:     [255;(TILEVOLUME*FIELDVOLUME)],
      rnd:     Rnd::new(),
      msg:     String::new(),
      topleft: Loc::default()
    }
  }

  fn memorySetCharColorMap(&mut self, m2c: &[(char,u8)]) {
    self.c2b = m2c.iter().map(|e|*e).collect::<HashMap<char,u8>>()
  }

  fn initializeMemory(&mut self, s: &[&str]) -> usize {
    let start = self.memPtr;
    s.iter().for_each(|s| {
        s.chars().for_each(|c| {
            self.memory[self.memPtr] = self.c2b[&c];
            self.memPtr += 1;
        })
    });
    start
  }

  fn initializeFieldData(&mut self, m: &[(char,u8)], s: &[&str]) {
    let hm = m.iter().map(|e|*e).collect::<HashMap<char,u8>>();
    let mut i = 0;
    s.iter().for_each(|s| {
        s.chars().for_each(|c| {
            self.field[i]=hm[&c];
            i += 1;
        })
    });
  }
  fn getFieldTile (&self, xf: usize, yf: usize) -> u8 {
    self.field[yf*FIELDWIDTH + xf]
  }
  fn getFieldTileMod (&self, xf: usize, yf: usize) -> u8 {
    self.field[(yf+FIELDHEIGHT)%FIELDHEIGHT*FIELDWIDTH + (xf+FIELDWIDTH)%FIELDWIDTH]
  }

  fn setFieldTile(&mut self, id: u8, loc: Loc) {
      self.field[loc.y*FIELDWIDTH+loc.x] = id
  }
  fn setSpriteLoc(&mut self, i: usize, loc: Loc) {
    self.sprites[i].loc = loc
  }
  fn setSpriteIdx(&mut self, i: usize, p: usize) {
    self.sprites[i].data = p;
  }
  fn rasterizeTilesSprites(&mut self, dataTiles: usize) {
    // tiles
    for fy in 0..FIELDHEIGHT {
    for fx in 0..FIELDWIDTH {
      if !self.update[fx+fy*FIELDWIDTH] { continue }
      self.update[fx+fy*FIELDWIDTH]=false;
      let roffset = fy*TILEVOLUME*FIELDWIDTH + fx*TILEWIDTH;
      let mut ptr = self.getFieldTile(fx, fy) as usize * TILEVOLUME + dataTiles;
      for cy in 0..TILEHEIGHT {
      for cx in 0..TILEWIDTH {
        self.raster [roffset + cx + cy*RASTERWIDTH] = self.memory [ptr];
        ptr += 1;
      }}
    }}

    // sprites
    for s in 0..SPRITECOUNT {
      if ! self.sprites[s].en { continue }

      let slocy = self.sprites[s].loc.y;
      let slocx = self.sprites[s].loc.x;

      // dirty tile bit
      let mut xsf = slocx/TILEWIDTH;
      let mut ysf = slocy/TILEHEIGHT;
      if FIELDWIDTH  <= xsf { xsf=FIELDWIDTH-1 }
      if FIELDHEIGHT <= ysf { ysf=FIELDHEIGHT-1 }
      self.update[ xsf               + ysf%FIELDHEIGHT*FIELDWIDTH] = true;
      self.update[(xsf+1)%FIELDWIDTH + ysf%FIELDHEIGHT*FIELDWIDTH] = true;
      self.update[(xsf+2)%FIELDWIDTH + ysf%FIELDHEIGHT*FIELDWIDTH] = true;

      self.update[ xsf               + (1+ysf)%FIELDHEIGHT*FIELDWIDTH] = true;
      self.update[(xsf+1)%FIELDWIDTH + (1+ysf)%FIELDHEIGHT*FIELDWIDTH] = true;
      self.update[(xsf+2)%FIELDWIDTH + (1+ysf)%FIELDHEIGHT*FIELDWIDTH] = true;

      self.update[ xsf               + (2+ysf)%FIELDHEIGHT*FIELDWIDTH] = true;
      self.update[(xsf+1)%FIELDWIDTH + (2+ysf)%FIELDHEIGHT*FIELDWIDTH] = true;
      self.update[(xsf+2)%FIELDWIDTH + (2+ysf)%FIELDHEIGHT*FIELDWIDTH] = true;

      let spritedata = self.sprites[s].data;

      for cy in 0..SPRITEHEIGHT {
          let y = (slocy + cy) % SPRITERASTERHEIGHT;
          if RASTERHEIGHT <= y { continue }
          for cx in 0..SPRITEWIDTH {
              let x = (slocx + cx) % SPRITERASTERWIDTH;
              if RASTERWIDTH <= x { continue }
              match self.memory[spritedata + cx+cy*SPRITEWIDTH] {
                0 => continue,
                b => self.raster[x+y*RASTERWIDTH] = b
              } // match
          } // for x
      } // for y
    } // for s

  }
  fn printField(&mut self, locCenter: Loc) {
    let w = min(self.term.w, RASTERWIDTH);
    let h = min(self.term.h, RASTERHEIGHT);
    let mut buff = String::new();
    let mut lastColor=0;
    write!(buff, "\x1b[H\x1b[30m").ok();
    let mut loc=(0,0);
    self.topleft = Loc{
      x: min(max(0, ((locCenter.x+16)%SPRITERASTERWIDTH -8-(w>>1))as isize), (RASTERWIDTH-w)  as isize) as usize,
      y: min(max(0, ((locCenter.y+16)%SPRITERASTERHEIGHT-8-(h>>1))as isize), (RASTERHEIGHT-h) as isize) as usize,
    };
    let mut idx = 0;
    let mut ridx = self.topleft.y*RASTERWIDTH + self.topleft.x;
    for y in 0..h {
      for x in 0..w {
        let b = self.raster[ridx];
        let l = self.last[idx];
        if b!=l {
          // Update cursor
          if loc != (x, y) { write!(buff, "\x1b[{};{}H", y+1,x+1).ok(); }
          if lastColor != b {
            if 0==b {
              write!(buff, " ").ok();
            } else {
              write!(buff, "\x1b[38;5;{}m{BLK}", b).ok();
              lastColor = b;
            }
          } else {
            if 0==b {
              write!(buff, " ").ok();
            } else {
              write!(buff, "{BLK}").ok();
            }
          }
          self.last[idx] = b;
          loc = (x+1, y);
        }
        ridx += 1;
        idx += 1;
      }
      ridx += RASTERWIDTH-w;
    }
    <Stdout as io::Write>::write_all(&mut stdout(), buff.as_bytes()).ok();
    print!("\x1b[H\x1b[37m{}\x1b[H", self.msg);
    self.msg.clear();
    //print!("\x1b[{};{}H\x1b[37m@", h/2, w/2);
  }
} // impl Graphics

////////////////////////////////////////

fn randir (vid: &mut Graphics, locField: Loc, lastdir: usize, go: usize) -> usize {
   let (xf, yf) = (locField.x, locField.y);
   if FIELDWIDTH <= xf || FIELDHEIGHT <= yf { return lastdir }
   let mut i = 0;
   let mut validDirs = [0; 4];
   let mut force = false;
   if lastdir!=3 && vid.getFieldTileMod(xf, yf-1)<3 { validDirs[i] = 0; i+=1; if go==0 { force=true } }
   if lastdir!=2 && vid.getFieldTileMod(xf-1, yf)<3 { validDirs[i] = 1; i+=1; if go==1 { force=true } }
   if lastdir!=1 && vid.getFieldTileMod(xf+1, yf)<3 { validDirs[i] = 2; i+=1; if go==2 { force=true } }
   if lastdir!=0 && vid.getFieldTileMod(xf, yf+1)<3 { validDirs[i] = 3; i+=1; if go==3 { force=true } }
   if force { return go }
   if 0 == i { return match lastdir { 0=>3, 3=>0, 1=>2, 2=>1, _=>lastdir } }
   validDirs[(vid.rnd.rnd() % i as u16) as usize]
}

fn dist (x: usize, y: usize, loc: Loc) -> usize {
  (x-loc.x)*(x-loc.x) + (y-loc.y)*(y-loc.y)
}

fn dirNew (vid: &mut Graphics, locField: Loc, lastdir: usize, locP:Loc, scared: bool) -> usize {
  let (x, y) = (locField.x, locField.y);
  if FIELDWIDTH <= x || FIELDHEIGHT <= y { return lastdir }
  let mut distance = IF!(scared,
      [(0,0),(0,1),(0,2),(0,3)],
      [(usize::MAX,0), (usize::MAX,1), (usize::MAX,2), (usize::MAX,3)]);
  let mut found=false;
  if lastdir!=3 && vid.getFieldTileMod(x, y-1)<3 { distance[0].0 = dist(x, y-1, locP); found=true; }
  if lastdir!=2 && vid.getFieldTileMod(x-1, y)<3 { distance[1].0 = dist(x-1, y, locP); found=true; }
  if lastdir!=1 && vid.getFieldTileMod(x+1, y)<3 { distance[2].0 = dist(x+1, y, locP); found=true; }
  if lastdir!=0 && vid.getFieldTileMod(x, y+1)<3 { distance[3].0 = dist(x, y+1, locP); found=true; }
  if !found { return match lastdir { 0=>3, 3=>0, 1=>2, 2=>1, _=>lastdir } } // reverse or keep going if nowhere to go
  distance.sort_by(|a,b|a.0.cmp(&b.0));
  distance[IF!(scared,3,0)].1
}

trait Entity {
  fn enable (&mut self, vid :&mut Graphics);
  fn tick (&mut self, vid: &mut Graphics);
}

const SPRITECENTERX: usize = (SPRITEWIDTH -TILEWIDTH)/2;
const SPRITECENTERY: usize = (SPRITEHEIGHT -TILEHEIGHT)/2;

const DMX :[usize; 4] = [0, usize::MAX, 1, 0];
const DMY :[usize; 4] = [usize::MAX, 0, 0, 1];

////////////////////////////////////////

struct Ghost {
    tick: usize,
    sprite: usize,
    data: usize,
    dataScared: usize,
    locField: Loc,
    dir: usize,
    locDesired: Loc,
    scared: bool,
}

impl Ghost {
  fn new (sprite: usize, data: usize, dataScared:usize, x:usize, y:usize, dir: usize) -> Ghost {
    Ghost{tick:0, sprite, data, dataScared, locField:Loc{x,y}, dir, locDesired:Loc::default(), scared:false}
  }
  fn setDesiredLoc (&mut self, loc: Loc) { self.locDesired = loc }
}

fn wrap (v: usize, m: usize) -> usize { (v+m)%m }

impl Entity for Ghost {
  fn enable (&mut self, vid :&mut Graphics) {
    vid.sprites[self.sprite].en = true;
  }
  fn tick (&mut self, vid: &mut Graphics) {
    if ! vid.sprites[self.sprite].en { return }
    let inc = self.tick % 8;
    vid.setSpriteLoc(self.sprite, Loc{
      x: wrap(TILEWIDTH*self.locField.x  + inc*DMX[self.dir] - SPRITECENTERX, SPRITERASTERWIDTH),
      y: wrap(TILEHEIGHT*self.locField.y + inc*DMY[self.dir] - SPRITECENTERY, SPRITERASTERHEIGHT)
    });

    // Ghost animation
    if self.scared { // scared or normal ghost
      vid.setSpriteIdx(self.sprite, self.dataScared + SPRITEVOLUME*(self.tick/8%2));
    } else {
      vid.setSpriteIdx(self.sprite, self.data+SPRITEVOLUME*(self.dir*2 + self.tick/8%2));
    }

    if 7 == inc {
      self.locField.x = (self.locField.x + DMX[self.dir] + SPRITEFIELDWIDTH) % SPRITEFIELDWIDTH;
      self.locField.y = (self.locField.y + DMY[self.dir] + SPRITEFIELDHEIGHT) % SPRITEFIELDHEIGHT;

      self.dir =
        if vid.rnd.rnd()&7 == 0 {
          randir(vid, self.locField, self.dir, 4)
        } else {
          dirNew(vid, self.locField, self.dir, self.locDesired, self.scared)
        };
    }

    self.tick += 1;
  }
}

////////////////////////////////////////

pub struct Pukman {
    tick: usize,
    sprite: usize,
    data: usize,
    dir: usize,
    locField: Loc,
    locRaster: Loc,
    go: usize,
    hugry: usize
}

impl Pukman {
  fn new (sprite: usize, data: usize, fx: usize, fy: usize, dir: usize) -> Pukman {
    Pukman{tick:0, sprite, data, dir, locField:Loc{x:fx, y:fy}, locRaster: Loc::default(), go:4, hugry:0}
  }
  fn hungry (&self) -> usize { self.hugry }
  fn go (&mut self, dir: usize) { self.go = dir }
}

impl Entity for Pukman {
  fn enable (&mut self, vid :&mut Graphics) {
    vid.sprites[self.sprite].en = true;
  }
  fn tick (&mut self, vid: &mut Graphics) {
    if ! vid.sprites[self.sprite].en { return }
    let mut inc = self.tick % 8;

    if self.go == [3,2,1,0][self.dir] && 0 < inc {
      self.locField.x = wrap(self.locField.x + DMX[self.dir], SPRITEFIELDWIDTH);
      self.locField.y = wrap(self.locField.y + DMY[self.dir], SPRITEFIELDHEIGHT);
      self.tick = 8 - inc;
      inc = self.tick % 8;
      self.dir = self.go;
      self.go = 4;
    }

    // tile-to-raster coordinates, center sprite on cell, parametric increment
    self.locRaster.x = wrap(TILEWIDTH*self.locField.x  + inc*DMX[self.dir] - SPRITECENTERX, SPRITERASTERWIDTH);
    self.locRaster.y = wrap(TILEHEIGHT*self.locField.y + inc*DMY[self.dir] - SPRITECENTERY, SPRITERASTERHEIGHT);
    vid.setSpriteLoc(self.sprite, self.locRaster);

    if 7 == inc { // on next call, sprite will be at this target loc, new valid diretion, 0==tick%8
       self.locField.x = wrap(self.locField.x + DMX[self.dir], SPRITEFIELDWIDTH);
       self.locField.y = wrap(self.locField.y + DMY[self.dir], SPRITEFIELDHEIGHT);
       self.dir = randir(vid, self.locField, self.dir, self.go);
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

    // disappear pill
    if 0 == self.tick%8 && self.locField.x < FIELDWIDTH && self.locField.y < FIELDHEIGHT {
        if 2 == vid.getFieldTile(self.locField.x, self.locField.y) {
          self.hugry = 512
        }
        vid.setFieldTile(0, self.locField);
    }
    if 0 < self.hugry { self.hugry -= 1 }

    self.tick += 1;
  }
} // impl Entity
 

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
      Ghost::new(1, offsets[0], offsets[4],  9, 20, 0),  //binky
      Ghost::new(2, offsets[1], offsets[4], 18, 20, 1),  //pinky
      Ghost::new(3, offsets[2], offsets[4],  9, 14, 2),  //inky
      Ghost::new(4, offsets[3], offsets[4], 18, 14, 3)]; //clyde
    let mut pukman = Pukman::new(0, offsets[5], 16, 20, 1);  // 13.5 26
    //pukman.go(2);
    let digitsMem = offsets[6];
    let dataTiles = offsets[7];

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
        let mut b :[u8;1] = [0];
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
        Ok(104)|Ok(97) =>  self.pukman.go(1), // Left: h a
        Ok(108)|Ok(100) => self.pukman.go(2), // Right: l d
        Ok(107)|Ok(119) => self.pukman.go(0), // Up: k w
        Ok(106)|Ok(115) => self.pukman.go(3), // Down: j s
        _ => ()
      }

      self.vid.msg = format!("{}", ["^", "<", ">", "v", "*"][self.pukman.go]); // DB dump direction

      self.pukman.tick(&mut self.vid);

      self.ghosts.iter_mut().for_each(|g| {
        g.scared = 0 < self.pukman.hungry();
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

      sleep(50);

      mark = SystemTime::now();
    }
    print!("\x1b[m");
  } // fn start

} // impl ArcadeGame

////////////////////////////////////////

fn main() {
  ArcadeGame::new(Graphics::new(Term::new())).start();
}
