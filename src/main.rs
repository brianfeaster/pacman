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
use data::{initializeVideoDataPukman};

mod util;
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

const TILEWIDTH: usize  = 8;
const TILEHEIGHT: usize = 8;
const TILEVOLUME: usize = TILEWIDTH*TILEHEIGHT;
const TILECOUNT: usize  = 128;

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

const SPRITEFIELDRASTERWIDTH: usize  = SPRITEFIELDWIDTH * TILEWIDTH;
const SPRITEFIELDRASTERHEIGHT: usize = SPRITEFIELDHEIGHT * TILEHEIGHT;

type TileData   = [u8; TILEVOLUME];
type SpriteData = [u8; SPRITEVOLUME];

#[derive(Clone,Copy,Default)]
struct Sprite {
    data: usize,
    x: usize,
    y: usize,
    en: bool,
}

pub struct Graphics {
    term: Term,
    tiledata: [TileData; TILECOUNT],
    spritedata: [SpriteData; TILECOUNT],
    field: [u8; FIELDVOLUME],
    update: [bool; FIELDVOLUME],
    sprites: [Sprite; SPRITECOUNT],
    raster: [u8; TILEVOLUME*FIELDVOLUME],
    last: [u8; TILEVOLUME*FIELDVOLUME],
    rnd: Rnd,
    msg: String,
    topleft: (usize, usize)
}

impl Graphics {
  fn new(term: Term) -> Graphics {
    print!("\x1bc\x1b[0;30;40m\x1b[H\x1b[J");
    Graphics{
      term,
      tiledata:  [[0; TILEVOLUME];   TILECOUNT],
      spritedata:[[0; SPRITEVOLUME]; TILECOUNT],
      sprites:   [Sprite::default(); SPRITECOUNT],
      field:     [63; FIELDVOLUME],
      update:    [true; FIELDVOLUME],
      raster:    [0;  (TILEVOLUME*FIELDVOLUME)],
      last:      [255;(TILEVOLUME*FIELDVOLUME)],
      rnd:       Rnd::new(),
      msg:       String::new(),
      topleft:   (20, 20)
    }
  }
  fn initializeTileData(&mut self, id: usize, m: &[(char,u8)], s: &[&str]) {
    let hm = m.iter().map(|e|*e).collect::<HashMap<char,u8>>();
    let mut i = 0;
    s.iter().for_each(|s| {
        s.chars().for_each(|c| {
            self.tiledata[id][i]=hm[&c];
            i += 1;
        })
    });
  }
  fn initializeSpriteData(&mut self, id: usize, m: &[(char,u8)], s: &[&str]) {
    let hm = m.iter().map(|e|*e).collect::<HashMap<char,u8>>();
    let mut i = 0;
    s.iter().for_each(|s| {
        s.chars().for_each(|c| {
            self.spritedata[id][i]=hm[&c];
            i += 1;
        })
    });
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

  pub fn setFieldTile(&mut self, id: u8, x: usize, y: usize) {
      self.field[y*FIELDWIDTH+x]=id
  }
  fn setSpriteLoc(&mut self, i: usize, x: usize, y: usize) {
    self.sprites[i].x = x;
    self.sprites[i].y = y;
  }
  fn setSpriteIdx(&mut self, i: usize, o: usize) {
    self.sprites[i].data = o;
  }
  fn rasterizeTilesSprites(&mut self) {
    // tiles
    let mut tilesUpdated = 0;
    for fy in 0..FIELDHEIGHT {
    for fx in 0..FIELDWIDTH {
      if !self.update[fx+fy*FIELDWIDTH] { continue }
      tilesUpdated += 1;
      self.update[fx+fy*FIELDWIDTH]=false;
      let roffset = fy*TILEVOLUME*FIELDWIDTH + fx*TILEWIDTH;
      let tiledata = self.tiledata [self.getFieldTile(fx, fy) as usize];
      for cy in 0..TILEHEIGHT {
      for cx in 0..TILEWIDTH {
        self.raster [roffset + cx + cy*RASTERWIDTH]
          = tiledata [cx + cy*TILEWIDTH]
      }}
    }}
    //write!(self.msg, "{:?}", tilesUpdated).ok();

    // sprites
    for s in 0..SPRITECOUNT {
      if ! self.sprites[s].en { continue }

      let slocy = self.sprites[s].y;
      let slocx = self.sprites[s].x;

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

      let spritedata = self.spritedata[self.sprites[s].data];

      for cy in 0..SPRITEHEIGHT {
          let y = (slocy + cy) % SPRITEFIELDRASTERHEIGHT;
          if RASTERHEIGHT <= y { continue }
          for cx in 0..SPRITEWIDTH {
              let x = (slocx + cx) % SPRITEFIELDRASTERWIDTH;
              if RASTERWIDTH <= x { continue }
              match spritedata[cx+cy*SPRITEWIDTH] {
                0 => continue,
                b => self.raster[x+y*RASTERWIDTH] = b
              } // match
          } // for x
      } // for y
    } // for s

  }
  fn printField(&mut self, (xr, yr): (usize, usize)) {
    let w = min(self.term.w, RASTERWIDTH);
    let h = min(self.term.h, RASTERHEIGHT);
    let mut buff = String::new();
    let mut lastColor=0;
    write!(buff, "\x1b[H\x1b[30m").ok();
    let mut loc=(0,0);
    let top = min(max(0, ((yr+16)%SPRITEFIELDRASTERHEIGHT-8-(h>>1))as isize), (RASTERHEIGHT-h) as isize) as usize;
    let left= min(max(0, ((xr+16)%SPRITEFIELDRASTERWIDTH -8-(w>>1))as isize), (RASTERWIDTH-w)  as isize) as usize;
    self.topleft = (top, left);
    let mut idx = 0;
    let mut ridx = top*RASTERWIDTH+left;
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
    //write!(buff, "\x1b[H{}        ", self.msg).ok();
    //self.msg.clear();
    <Stdout as io::Write>::write_all(&mut stdout(), buff.as_bytes()).ok();
    //print!("\x1b[{};{}H\x1b[37m@", h/2, w/2);
  }
  fn getTopLeft (&self) -> (usize, usize) { self.topleft }
} // impl Graphics

////////////////////////////////////////

fn randir (vid: &mut Graphics, xf: usize, yf: usize, lastdir: usize, go: usize) -> usize {
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

fn dist (x: usize, y: usize, pxy: (usize, usize)) -> usize {
  (x-pxy.0)*(x-pxy.0) + (y-pxy.1)*(y-pxy.1)
}

fn dirchase (vid: &mut Graphics, x: usize, y: usize, lastdir: usize, xyp:( usize,  usize)) -> usize {
  if FIELDWIDTH <= x || FIELDHEIGHT <= y { return lastdir }
  let mut i = 0;
  let mut distance = [usize::MAX; 4];
  if lastdir!=3 && vid.getFieldTileMod(x, y-1)<3 { distance[0] = dist(x, y-1, xyp); i+=1; }
  if lastdir!=2 && vid.getFieldTileMod(x-1, y)<3 { distance[1] = dist(x-1, y, xyp); i+=1; }
  if lastdir!=1 && vid.getFieldTileMod(x+1, y)<3 { distance[2] = dist(x+1, y, xyp); i+=1; }
  if lastdir!=0 && vid.getFieldTileMod(x, y+1)<3 { distance[3] = dist(x, y+1, xyp); i+=1; }
  if 0 == i { return match lastdir { 0=>3, 3=>0, 1=>2, 2=>1, _=>lastdir } } // reverse if nother option

  let mut dist = distance[0];
  let mut d = 0;

  if distance[1]<dist { dist=distance[1]; d=1; }
  if distance[2]<dist { dist=distance[2]; d=2; }
  if distance[3]<dist { d=3; }
  d
}

trait Entity {
  fn enable (&mut self, vid :&mut Graphics);
  fn tick (&mut self, vid: &mut Graphics);
  fn loc (&self) -> (usize, usize) { (0,0) }
  fn locr (&self) -> (usize, usize) { (0,0) }
  fn go (&mut self, _dir: usize) { }
  fn locset (&mut self, _l: &(usize, usize)) { }
  fn getTick (&self) -> usize { 0 }
}

const DMX :[usize; 4] = [0, usize::MAX, 1, 0];
const DMY :[usize; 4] = [usize::MAX, 0, 0, 1];

////////////////////////////////////////

struct Ghost {
    sprite: usize,
    data: usize,
    fx: usize,
    fy: usize,
    dir: usize,
    tick: usize,
    xr: usize,
    yr: usize,
}

impl Ghost {
  fn new (sprite: usize, data: usize, fx: usize, fy: usize, dir: usize) -> Box<impl Entity> {
    Box::new(Ghost{sprite, data, fx, fy, dir, tick:0, xr:0, yr:0})
  }
}


impl Entity for Ghost {
  fn enable (&mut self, vid :&mut Graphics) {
    vid.sprites[self.sprite].en = true;
  }
  fn tick (&mut self, vid: &mut Graphics) {
    if ! vid.sprites[self.sprite].en { return }
    let inc = self.tick % 8;
    let xr = (TILEWIDTH*self.fx  - (SPRITEWIDTH - TILEWIDTH)/2   + inc*DMX[self.dir] + SPRITEFIELDRASTERWIDTH) % SPRITEFIELDRASTERWIDTH;
    let yr = (TILEHEIGHT*self.fy - (SPRITEHEIGHT - TILEHEIGHT)/2 + inc*DMY[self.dir] + SPRITEFIELDRASTERHEIGHT) % SPRITEFIELDRASTERHEIGHT;
    vid.setSpriteLoc(self.sprite, xr, yr);
    // Ghost animation
    vid.setSpriteIdx(self.sprite, self.data+self.dir*2 + self.tick/8%2);
    if 7 == inc {
      self.fx = (self.fx + DMX[self.dir] + SPRITEFIELDWIDTH) % SPRITEFIELDWIDTH;
      self.fy = (self.fy + DMY[self.dir] + SPRITEFIELDHEIGHT) % SPRITEFIELDHEIGHT;
      self.dir = if vid.rnd.rnd()&7 == 0 {
        randir(vid, self.fx, self.fy, self.dir, 4)
      } else {
        dirchase(vid, self.fx, self.fy, self.dir, (self.xr, self.yr))
      };
    }
    self.xr = xr;
    self.yr = yr;
    self.tick += 1;
  }
  fn locset (&mut self, l: &(usize, usize)) {
    self.xr = l.0;
    self.yr = l.1;
  }
}

////////////////////////////////////////

pub struct Pukman {
    sprite: usize,
    data: usize,
    fx: usize,
    fy: usize,
    dir: usize,
    tick: usize,
    xr: usize, // raster screen position
    yr: usize,
    go: usize,
}

impl Pukman {
  fn new (sprite: usize, data: usize, fx: usize, fy: usize, dir: usize) -> Box<impl Entity> {
    Box::new(Self{sprite, data, fx, fy, dir, tick:0, xr:0, yr:0, go:4})
  }
}

impl Entity for Pukman {
  fn enable (&mut self, vid :&mut Graphics) {
    vid.sprites[self.sprite].en = true;
  }
  fn tick (&mut self, vid: &mut Graphics) {
    if ! vid.sprites[self.sprite].en { return }
    let inc = self.tick % 8;

    // tile to raster coordinates, center sprite on cell, parametric increment
    let xr = (TILEWIDTH*self.fx  - (SPRITEWIDTH-TILEWIDTH)/2   + inc*DMX[self.dir] + SPRITEFIELDRASTERWIDTH) % SPRITEFIELDRASTERWIDTH;
    let yr = (TILEHEIGHT*self.fy - (SPRITEHEIGHT-TILEHEIGHT)/2 + inc*DMY[self.dir] + SPRITEFIELDRASTERHEIGHT) % SPRITEFIELDRASTERHEIGHT;
    vid.setSpriteLoc(self.sprite, xr, yr);
    if 7 == inc {
       self.fx = (self.fx + DMX[self.dir] + SPRITEFIELDWIDTH) % SPRITEFIELDWIDTH;
       self.fy = (self.fy + DMY[self.dir] + SPRITEFIELDHEIGHT) % SPRITEFIELDHEIGHT;
       self.dir = randir(vid, self.fx, self.fy, self.dir, self.go);
    }
    // Pukman animation
    vid.setSpriteIdx(self.sprite,
      self.data + match self.tick%4 {
        0 => 0,
        1 => self.dir * 2 + 1,
        2 => self.dir * 2 + 2,
        3 => self.dir * 2 + 1,
        _ => 0
      }
    );
    // disappear pill
    if 0 == self.tick%8 && self.fx < FIELDWIDTH && self.fy < FIELDHEIGHT { vid.setFieldTile(0, self.fx, self.fy); }
    self.xr = xr;
    self.yr = yr;
    self.tick += 1;
  }
  fn loc (&self) -> (usize, usize) { (self.fx, self.fy) }
  fn locr (&self) -> (usize, usize) { (self.xr, self.yr) }
  fn go (&mut self, dir: usize) { self.go = dir }
  fn getTick (&self) -> usize { self.tick }
}

////////////////////////////////////////

struct ArcadeGame {
  vid: Graphics,
  entities: [Box<dyn Entity>; 5],
  buttons: Receiver<u8>,
}

impl ArcadeGame {
  fn new (mut vid: Graphics) -> Self {
    let (tx, rx) = channel::<u8>();
    let mut si = stdin();
    thread::spawn(move || loop {
        let mut b :[u8;1] = [0];
        si.read(&mut b).unwrap();
        tx.send(b[0]).unwrap();
    });
    initializeVideoDataPukman(&mut vid);
    let mut entities: [Box<dyn Entity>; 5] = [
      Pukman::new(0, 32, 16, 20, 1),  // 13.5 26
      Ghost::new(1,   0,  9, 20, 0),  //binky
      Ghost::new(2,   8, 18, 20, 1),  //pinky
      Ghost::new(3,  16,  9, 14, 2),  //inky
      Ghost::new(4,  24, 18, 14, 3)]; //clyde
    entities.iter_mut().for_each(|e| e.enable(&mut vid));
    vid.sprites[5].en = true;
    vid.sprites[6].en = true;
    vid.sprites[7].en = true;
    vid.sprites[8].en = true;
    vid.sprites[9].en = true;
    ArcadeGame{vid, entities, buttons:rx}
  }
  fn start(&mut self) {
    let mut now = SystemTime::now();
    let mut dur = now.elapsed().unwrap().as_micros() as usize;
    const fpsSamps: usize = 50;
    let mut fps: [usize; fpsSamps] = [0; fpsSamps];
    let mut fpsCount=0;
    let mut fpsp=0;
    let mut sum=0;
    loop {
      match self.buttons.try_recv() {
        Ok(113)|Ok(27)|Ok(81)|Ok(3) => break, // q ESC ^C
        Ok(104)|Ok(97) => self.entities[0].go(1), // h a
        Ok(108)|Ok(100) => self.entities[0].go(2),// l d
        Ok(107)|Ok(119) => self.entities[0].go(0),// k w
        Ok(106)|Ok(115) => self.entities[0].go(3),// j s
        _ => ()
      }

      let pm = &mut self.entities[0];
      pm.tick(&mut self.vid);

      let (xpm, ypm) = pm.locr();
      let (top, left) = self.vid.getTopLeft();
      self.vid.sprites[5].x = left+32;
      self.vid.sprites[5].y = top+1;
      self.vid.sprites[5].data=50+dur%10;

      self.vid.sprites[6].x = left+24;
      self.vid.sprites[6].y = top+1;
      self.vid.sprites[6].data=50+dur/10%10;

      self.vid.sprites[7].x = left+16;
      self.vid.sprites[7].y = top+1;
      self.vid.sprites[7].data=50+dur/100%10;

      self.vid.sprites[8].x = left+8;
      self.vid.sprites[8].y = top+1;
      self.vid.sprites[8].data=50+dur/1000%10;

      self.vid.sprites[9].x = left;
      self.vid.sprites[9].y = top+1;
      self.vid.sprites[9].data=50+dur/10000%10;

      let pml = pm.loc();
      self.entities.iter_mut().skip(1).for_each(|e| e.locset(&pml));
      self.entities.iter_mut().skip(1).for_each(|e| e.tick(&mut self.vid));

      self.vid.rasterizeTilesSprites();
      self.vid.printField(self.entities[0].locr());

      dur = now.elapsed().unwrap().as_micros() as usize;
      sum = sum - fps[fpsp] + dur;
      fps[fpsp]=dur;
      fpsp=(fpsp+1)%fpsSamps;
      if fpsCount != fpsSamps { fpsCount += 1 }
      dur = 1000000/(sum / fpsCount) as usize;
      if 99999 < dur { dur = 99999; }

      sleep(10);

      now = SystemTime::now();
    }
    print!("\x1b[m");
  }
}

////////////////////////////////////////

fn main() {
  ArcadeGame::new(Graphics::new(Term::new())).start();
}
