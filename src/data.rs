use crate::Graphics;


fn writeGhost2048b (vid: &mut Graphics) -> usize {
  vid.initializeMemory(&[
    // right 0
    "                ",
    "      aaaa      ",
    "    aaaaaaaa    ",
    "   aaaaaaaaaa   ",
    "  aaa..aaaa..a  ",
    "  aa....aa....  ",
    "  aa..##aa..##  ",
    " aaa..##aa..##a ",
    " aaaa..aaaa..aa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aa aaa  aaa aa ",
    " a   aa  aa   a ",
    "                ",
    // right 1
    "                ",
    "      aaaa      ",
    "    aaaaaaaa    ",
    "   aaaaaaaaaa   ",
    "  aaa..aaaa..a  ",
    "  aa....aa....  ",
    "  aa..##aa..##  ",
    " aaa..##aa..##a ",
    " aaaa..aaaa..aa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaa aaaa aaaa ",
    "  aa   aa   aa  ",
    "                ",
    // down 0
    "                ",
    "      aaaa      ",
    "    aaaaaaaa    ",
    "   aaaaaaaaaa   ",
    "  aaaaaaaaaaaa  ",
    "  aa..aaaa..aa  ",
    "  a....aa....a  ",
    " aa....aa....aa ",
    " aa.##.aa.##.aa ",
    " aaa##aaaa##aaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aa aaa  aaa aa ",
    " a   aa  aa   a ",
    "                ",
    // down 1
    "                ",
    "      aaaa      ",
    "    aaaaaaaa    ",
    "   aaaaaaaaaa   ",
    "  aaaaaaaaaaaa  ",
    "  aa..aaaa..aa  ",
    "  a....aa....a  ",
    " aa....aa....aa ",
    " aa.##.aa.##.aa ",
    " aaa##aaaa##aaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaa aaaa aaaa ",
    "  aa   aa   aa  ",
    "                ",
    // left 0
    "                ",
    "      aaaa      ",
    "    aaaaaaaa    ",
    "   aaaaaaaaaa   ",
    "  a..aaaa..aaa  ",
    "  ....aa....aa  ",
    "  ##..aa##..aa  ",
    " a##..aa##..aaa ",
    " aa..aaaa..aaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aa aaa  aaa aa ",
    " a   aa  aa   a ",
    "                ",
    // left 1
    "                ",
    "      aaaa      ",
    "    aaaaaaaa    ",
    "   aaaaaaaaaa   ",
    "  a..aaaa..aaa  ",
    "  ....aa....aa  ",
    "  ##..aa##..aa  ",
    " a##..aa##..aaa ",
    " aa..aaaa..aaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaa aaaa aaaa ",
    "  aa   aa   aa  ",
    "                ",
    // up 0
    "                ",
    "      aaaa      ",
    "    ##aaaa##    ",
    "   .##.aa.##.   ",
    "  a....aa....a  ",
    "  a....aa....a  ",
    "  aa..aaaa..aa  ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aa aaa  aaa aa ",
    " a   aa  aa   a ",
    "                ",
    // up 1
    "                ",
    "      aaaa      ",
    "    ##aaaa##    ",
    "   .##.aa.##.   ",
    "  a....aa....a  ",
    "  a....aa....a  ",
    "  aa..aaaa..aa  ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaaaaaaaaaaaa ",
    " aaaa aaaa aaaa ",
    "  aa   aa   aa  ",
    "                ",])
} // writeGhost2048b


fn writeScaredGhost512b (vid: &mut Graphics) -> usize {
  vid.memorySetCharColorMap(&[(' ',0), ('#',4), ('.',7)]);
  vid.initializeMemory(&[
    // scared ghost 0
    "                ",
    "      ####      ",
    "    ########    ",
    "   ##########   ",
    "  ############  ",
    "  ###..##..###  ",
    "  ###..##..###  ",
    " ############## ",
    " ############## ",
    " ##..##..##..## ",
    " #.##..##..##.# ",
    " ############## ",
    " ############## ",
    " ## ###  ### ## ",
    " #   ##  ##   # ",
    "                ",
    // scared ghost 0
    "                ",
    "      ####      ",
    "    ########    ",
    "   ##########   ",
    "  ############  ",
    "  ###..##..###  ",
    "  ###..##..###  ",
    " ############## ",
    " ############## ",
    " ##..##..##..## ",
    " #.##..##..##.# ",
    " ############## ",
    " ############## ",
    " #### #### #### ",
    "  ##   ##   ##  ",
    "                ",])
} // writeScaredGhost512b

fn writeGhostEyes1024b (vid: &mut Graphics) -> usize {
  vid.memorySetCharColorMap(&[(' ',0), ('#',4), ('.',7)]);
  vid.initializeMemory(&[
    // right
    "                ",
    "                ",
    "                ",
    "    ..    ..    ",
    "   ....  ....   ",
    "   ..##  ..##   ",
    "   ..##  ..##   ",
    "    ..    ..    ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    // down
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "    ..    ..    ",
    "   ....  ....   ",
    "   ....  ....   ",
    "   .##.  .##.   ",
    "    ##    ##    ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    // left
    "                ",
    "                ",
    "                ",
    "    ..    ..    ",
    "   ....  ....   ",
    "   ##..  ##..   ",
    "   ##..  ##..   ",
    "    ..    ..    ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    // up
    "                ",
    "                ",
    "    ##    ##    ",
    "   .##.  .##.   ",
    "   ....  ....   ",
    "   ....  ....   ",
    "    ..    ..    ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",])
}

fn writePukman2304b (vid: &mut Graphics) -> usize {
  vid.memorySetCharColorMap(&[(' ',0), ('o',11)]);
  vid.initializeMemory(&[
    "                ",
    "     ooooo      ",
    "   ooooooooo    ",
    "  ooooooooooo   ",
    "  ooooooooooo   ",
    " ooooooooooooo  ",
    " ooooooooooooo  ",
    " ooooooooooooo  ",
    " ooooooooooooo  ",
    " ooooooooooooo  ",
    "  ooooooooooo   ",
    "  ooooooooooo   ",
    "   ooooooooo    ",
    "     ooooo      ",
    "                ",
    "                ",
    //
    "                ",
    "     ooooo      ",
    "   ooooooooo    ",
    "  ooooooooooo   ",
    "  ooooooooooo   ",
    " oooooooooo     ",
    " ooooooo        ",
    " oooo           ",
    " ooooooo        ",
    " oooooooooo     ",
    "  ooooooooooo   ",
    "  ooooooooooo   ",
    "   ooooooooo    ",
    "     ooooo      ",
    "                ",
    "                ",
    //
    "                ",
    "     ooooo      ",
    "   ooooooo      ",
    "  ooooooo       ",
    "  oooooo        ",
    " oooooo         ",
    " ooooo          ",
    " oooo           ",
    " ooooo          ",
    " oooooo         ",
    "  oooooo        ",
    "  ooooooo       ",
    "   ooooooo      ",
    "     ooooo      ",
    "                ",
    "                ",
    //
    "                ",
    "     ooooo      ",
    "   ooooooooo    ",
    "  ooooooooooo   ",
    "  ooooooooooo   ",
    " oooooo oooooo  ",
    " oooooo oooooo  ",
    " oooooo oooooo  ",
    " ooooo   ooooo  ",
    " ooooo   ooooo  ",
    "  oooo   oooo   ",
    "  ooo     ooo   ",
    "   oo     oo    ",
    "                ",
    "                ",
    "                ",
    //
    "                ",
    "     ooooo      ",
    "   ooooooooo    ",
    "  ooooooooooo   ",
    "  ooooooooooo   ",
    " oooooo oooooo  ",
    " ooooo   ooooo  ",
    " oooo     oooo  ",
    " ooo       ooo  ",
    " oo         oo  ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    //
    "                ",
    "     ooooo      ",
    "   ooooooooo    ",
    "  ooooooooooo   ",
    "  ooooooooooo   ",
    "    oooooooooo  ",
    "       ooooooo  ",
    "          oooo  ",
    "       ooooooo  ",
    "    oooooooooo  ",
    "  ooooooooooo   ",
    "  ooooooooooo   ",
    "   ooooooooo    ",
    "     ooooo      ",
    "                ",
    "                ",
    //
    "                ",
    "     ooooo      ",
    "     ooooooo    ",
    "     oooooooo   ",
    "       oooooo   ",
    "        oooooo  ",
    "         ooooo  ",
    "          oooo  ",
    "         ooooo  ",
    "        oooooo  ",
    "       oooooo   ",
    "      ooooooo   ",
    "     ooooooo    ",
    "     ooooo      ",
    "                ",
    "                ",
    //
    "                ",
    "                ",
    "   oo     oo    ",
    "  ooo     ooo   ",
    "  oooo   oooo   ",
    " ooooo   ooooo  ",
    " ooooo   ooooo  ",
    " oooooo oooooo  ",
    " oooooo oooooo  ",
    " oooooo oooooo  ",
    "  ooooooooooo   ",
    "  ooooooooooo   ",
    "   ooooooooo    ",
    "     ooooo      ",
    "                ",
    "                ",
    //
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    " oo         oo  ",
    " ooo       ooo  ",
    " oooo     oooo  ",
    " ooooo   ooooo  ",
    " oooooo oooooo  ",
    "  ooooooooooo   ",
    "  ooooooooooo   ",
    "   ooooooooo    ",
    "     ooooo      ",
    "                ",
    "                ",])
}


fn writeDigits640b (vid: &mut Graphics) -> usize {
  vid.memorySetCharColorMap(&[(' ',0), ('x',11)]);
  vid.initializeMemory(&[
    //
    "  xxxx          ",
    " xx  xx         ",
    " xx xxx         ",
    " xxx xx         ",
    " xx  xx         ",
    " xx  xx         ",
    "  xxxx          ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    //
    "   xx           ",
    "   xx           ",
    "  xxx           ",
    "   xx           ",
    "   xx           ",
    "   xx           ",
    " xxxxxx         ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    //
    "  xxxx          ",
    " xx  xx         ",
    "     xx         ",
    "    xx          ",
    "  xx            ",
    " xx             ",
    " xxxxxx         ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    //
    "  xxxx          ",
    " xx  xx         ",
    "     xx         ",
    "   xxx          ",
    "     xx         ",
    " xx  xx         ",
    "  xxxx          ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    //
    "     xx         ",
    "    xxx         ",
    "   xxxx         ",
    " xx  xx         ",
    " xxxxxxx        ",
    "     xx         ",
    "     xx         ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    //
    " xxxxxx         ",
    " xx             ",
    " xxxxx          ",
    "     xx         ",
    "     xx         ",
    " xx  xx         ",
    "  xxxx          ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    //
    "  xxxx          ",
    " xx  xx         ",
    " xx             ",
    " xxxxx          ",
    " xx  xx         ",
    " xx  xx         ",
    "  xxxx          ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    //
    " xxxxxx         ",
    " xx  xx         ",
    "    xx          ",
    "   xx           ",
    "   xx           ",
    "   xx           ",
    "   xx           ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    //
    "  xxxx          ",
    " xx  xx         ",
    " xx  xx         ",
    "  xxxx          ",
    " xx  xx         ",
    " xx  xx         ",
    "  xxxx          ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    //
    "  xxxx          ",
    " xx  xx         ",
    " xx  xx         ",
    "  xxxxx         ",
    "     xx         ",
    " xx  xx         ",
    "  xxxx          ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    // small digits
    " xx             ",
    "x  x            ",
    "x  x            ",
    " xx             ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    //
    " xx             ",
    " xx             ",
    " xx             ",
    " xx             ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    //
    "xxxx            ",
    "  xx            ",
    "xx              ",
    "xxxx            ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    //
    "xxxx            ",
    "  xx            ",
    " xxx            ",
    "xxxx            ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    //
    "x  x            ",
    "xxxx            ",
    "   x            ",
    "   x            ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    //
    "xxxx            ",
    "xx              ",
    "  xx            ",
    "xxx             ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    //
    " xx             ",
    "x               ",
    "xxxx            ",
    " xx             ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    //
    "xxxx            ",
    "   x            ",
    "  x             ",
    "  x             ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    //
    "xxxx            ",
    "xx x            ",
    "x xx            ",
    "xxxx            ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    //
    "xxxx            ",
    "xxxx            ",
    "   x            ",
    "   x            ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
    "                ",
  ])
}

fn writeIgnignok (vid: &mut Graphics) -> usize {
  vid.memorySetCharColorMap(&[(' ',0), ('g',2), ('b',4), ('.',4)]);
  vid.initializeMemory(&[
    "                ",
    "    gg     ggg  ",
    "    ggg    ggg  ",
    "  ggggg gggggg  ",
    "  ggg.gggg.ggg  ",
    "  gg.gggggg.gg  ",
    "  g.g..gg..g.g  ",
    "   ggggggggggg  ",
    "   ggggggggggg b",
    "   ggggggggggg b",
    " bggg......gggb ",
    "bgggggggggggggg ",
    "bgggggggggggggg ",
    "     b    b     ",
    "   bbb    bbb   ",
    "                ",

    "                ",
    "    gg     ggg  ",
    "    ggg    ggg  ",
    "  ggggg gggggg  ",
    "  ggg.gggg.ggg  ",
    "  gg.gggggg.gg  ",
    "  g.g..gg..g.g  ",
    "   ggggggggggg  ",
    "b  ggggggggggg  ",
    "b  ggggggggggg  ",
    " bggg......gggb ",
    " ggggggggggggggb",
    " ggggggggggggggb",
    "     b    b     ",
    "   bbb    bbb   ",
    "                ",
  ])
}
fn _writeDrinkyBird (vid: &mut Graphics) -> usize {
  vid.memorySetCharColorMap(&[(' ',0), ('g',2), ('b',4), ('r',1),('h',7),('w',15)]);
  vid.initializeMemory(&[
    "    b           ",
    "   bbbb         ",
    "   bbrrr        ",
    "    brrr        ",
    "    r  h        ",
    "   r    h       ",
    "       wwh      ",
    "       ww h     ",
    "       ww  h g  ",
    "       ww   gg  ",
    "        ww  gg  ",
    "        ww      ",
    "         w      ",
    "         rr     ",
    "      r  rr     ",
    "      rrrrr     ",

    "                ",
    "                ",
    "                ",
    "                ",
    " b           g  ",
    "bbrr          g ",
    "bbrrhhhhhhhhhhgg",
    " br    ww     gg",
    "  r    ww       ",
    "       ww       ",
    "        ww      ",
    "        ww      ",
    "         w      ",
    "         rr     ",
    "      r  rr     ",
    "      rrrrr     ",
  ])
}

fn writeTiles (vid: &mut Graphics) -> usize {
  vid.memorySetCharColorMap(&[(' ',0), ('@',4), ('*',7), ('#',11), ('-',13)]);
  let offset = vid.initializeMemory(&[
    "        ", // 0
    "        ",
    "        ",
    "        ",
    "        ",
    "        ",
    "        ",
    "        ",

    "        ", // 1
    "        ",
    "        ",
    "   **   ",
    "   **   ",
    "        ",
    "        ",
    "        ",

    "  ####  ", // 2
    " ###### ",
    "########",
    "########",
    "########",
    "########",
    " ###### ",
    "  ####  ",

    "        ", // 3
    "        ",
    "        ",
    "        ",
    "        ",
    "--------",
    "--------",
    "        ",

    "    @@@@", // 4
    "  @@    ",
    " @      ",
    " @   @@@",
    "@   @   ",
    "@  @    ",
    "@  @    ",
    "@  @    ",

    "@@@@    ", // 5
    "    @@  ",
    "      @ ",
    "@@@   @ ",
    "   @   @",
    "    @  @",
    "    @  @",
    "    @  @",

    "@  @    ", // 6
    "@  @    ",
    "@  @    ",
    "@   @   ",
    " @   @@@",
    " @      ",
    "  @@    ",
    "    @@@@",

    "    @  @", // 7
    "    @  @",
    "    @  @",
    "   @   @",
    "@@@   @ ",
    "      @ ",
    "    @@  ",
    "@@@@    ",

    "@@@@@@@@", // 8
    "        ",
    "        ",
    "     @@@",
    "    @   ",
    "   @    ",
    "   @    ",
    "   @    ",

    "@@@@@@@@", // 9
    "        ",
    "        ",
    "@@@     ",
    "   @    ",
    "    @   ",
    "    @   ",
    "    @   ",

    "   @    ", // 10
    "   @    ",
    "   @    ",
    "    @   ",
    "     @@@",
    "        ",
    "        ",
    "@@@@@@@@",

    "    @   ", // 11
    "    @   ",
    "    @   ",
    "   @    ",
    "@@@     ",
    "        ",
    "        ",
    "@@@@@@@@",

    "@       ", // 12
    "@       ",
    "@       ",
    "@    @@@",
    "@   @   ",
    "@  @    ",
    "@  @    ",
    "@  @    ",

    "       @", // 13
    "       @",
    "       @",
    "@@@    @",
    "   @   @",
    "    @  @",
    "    @  @",
    "    @  @",

    "@  @    ", // 14
    "@  @    ",
    "@  @    ",
    "@   @   ",
    "@    @@@",
    "@       ",
    "@       ",
    "@       ",

    "    @  @", // 15
    "    @  @",
    "    @  @",
    "   @   @",
    "@@@    @",
    "       @",
    "       @",
    "       @",

    "@@@@@@@@", // 16
    "        ",
    "        ",
    "@@@@@@@@",
    "        ",
    "        ",
    "        ",
    "        ",

    "@  @    ", // 17
    "@  @    ",
    "@  @    ",
    "@  @    ",
    "@  @    ",
    "@  @    ",
    "@  @    ",
    "@  @    ",

    "    @  @", // 18
    "    @  @",
    "    @  @",
    "    @  @",
    "    @  @",
    "    @  @",
    "    @  @",
    "    @  @",

    "        ", // 19
    "        ",
    "        ",
    "        ",
    "@@@@@@@@",
    "        ",
    "        ",
    "@@@@@@@@",

    "        ", // 20
    "        ",
    "        ",
    "        ",
    "      @@",
    "     @  ",
    "    @   ",
    "    @   ",

    "        ", // 21
    "        ",
    "        ",
    "        ",
    "@@      ",
    "  @     ",
    "   @    ",
    "   @    ",

    "    @   ", // 22
    "    @   ",
    "     @  ",
    "      @@",
    "        ",
    "        ",
    "        ",
    "        ",

    "   @    ", // 23
    "   @    ",
    "  @     ",
    "@@      ",
    "        ",
    "        ",
    "        ",
    "        ",

    "        ", // 24
    "        ",
    "        ",
    "     @@@",
    "    @   ",
    "   @    ",
    "   @    ",
    "   @    ",

    "        ", // 25
    "        ",
    "        ",
    "@@@     ",
    "   @    ",
    "    @   ",
    "    @   ",
    "    @   ",

    "   @    ", // 26
    "   @    ",
    "   @    ",
    "    @   ",
    "     @@@",
    "        ",
    "        ",
    "        ",

    "    @   ", // 27
    "    @   ",
    "    @   ",
    "   @    ",
    "@@@     ",
    "        ",
    "        ",
    "        ",

    "        ", // 28
    "        ",
    "        ",
    "        ",
    "@@@@@@@@",
    "        ",
    "        ",
    "        ",

    "    @   ", // 29
    "    @   ",
    "    @   ",
    "    @   ",
    "    @   ",
    "    @   ",
    "    @   ",
    "    @   ",

    "   @    ", // 30
    "   @    ",
    "   @    ",
    "   @    ",
    "   @    ",
    "   @    ",
    "   @    ",
    "   @    ",

    "        ", // 31
    "        ",
    "        ",
    "@@@@@@@@",
    "        ",
    "        ",
    "        ",
    "        ",

    "        ", // 32
    "        ",
    "        ",
    "        ",
    "    @@@@",
    "    @   ",
    "    @   ",
    "    @  @",

    "        ", // 33
    "        ",
    "        ",
    "        ",
    "@@@@    ",
    "   @    ",
    "   @    ",
    "@  @    ",

    "    @  @", // 34
    "    @   ",
    "    @   ",
    "    @@@@",
    "        ",
    "        ",
    "        ",
    "        ",

    "@  @    ", // 35
    "   @    ",
    "   @    ",
    "@@@@    ",
    "        ",
    "        ",
    "        ",
    "        ",
  ]);
  offset
}


pub fn initializeVideoDataPukman (vid: &mut Graphics) -> [usize; 10] {

  let mut offsets: [usize; 10] = [0; 10];

  vid.memorySetCharColorMap(&[(' ',0), ('#',4), ('a',9),  ('.',15)]);
  offsets[0] = writeGhost2048b(vid);

  vid.memorySetCharColorMap(&[(' ',0), ('#',4), ('a',13), ('.',15)]);
  offsets[1] = writeGhost2048b(vid);

  vid.memorySetCharColorMap(&[(' ',0), ('#',4), ('a',14), ('.',15)]);
  offsets[2] = writeGhost2048b(vid);

  vid.memorySetCharColorMap(&[(' ',0), ('#',4), ('a',3),  ('.',15)]);
  offsets[3] = writeGhost2048b(vid);

  offsets[4] = writeScaredGhost512b(vid);
  offsets[5] = writeGhostEyes1024b(vid);
  offsets[6] = writePukman2304b(vid);
  //offsets[7] = writeDrinkyBird(vid);
  offsets[7] = writeIgnignok(vid);
  offsets[8] = writeDigits640b(vid);
  offsets[9] = writeTiles(vid);


  vid.initializeFieldData(&[
    // nothing pill power-pill
    (' ',0), ('.',1), ('*',2),

    // double corners
    ('A',4), ('B',5), ('C',6), ('D',7),
    ('E',8), ('F',9), ('G',10), ('H',11),
    ('I',12),('J',13),('K',14),('L',15),
    // double edges
    ('M',16), ('N',17), ('O',18), ('P',19),

    // corner
    ('a',20), ('b',21), ('c',22), ('d',23),
    // corner inside
    ('e',24), ('f',25), ('g',26), ('h',27),
    // edge
    ('i',28),('j',29),('k',30),('l',31),
    // double hard corner
    ('m',32),('n',33),('o',34),('p',35),
    // door
    ('q',3)], &[
    // 28x36 = 1008
    "           N jk O           ",
    "AMMMMMMMMMMd cd cMMMMMMMMMMB",
    "N............  ............O",
    "N.aiib.aiiib.ab.aiiib.aiib.O",
    "N*j  k.j   k.jk.j   k.j  k*O",
    "N.clld.cllld.cd.cllld.clld.O",
    "N..........................O",
    "N.aiib.ab.aiiiiiib.ab.aiib.O",
    "N.clld.jk.cllfelld.jk.clld.O",
    "N......jk....jk....jk......O",
    "CPPPPb.jgiib jk aiihk.aPPPPD",
    "     N.jelld cd cllfk.O     ",
    "     N.jk          jk.O     ",
    "     N.jk mPPqqPPn jk.O     ",
    "MMMMMd.cd O      N cd.cMMMMMMM",
    "      .   O      N   .      ",
    "PPPPPb.ab O      N ab.aPPPPPPP",
    "     N.jk oMMMMMMp jk.O     ",
    "     N.jk          jk.O     ",
    "     N.jk aiiiiiib jk.O     ",
    "AMMMMd.cd cllfelld cd.cMMMMB",
    "N............jk............O",
    "N.aiib.aiiib.jk.aiiib.aiib.O",
    "N.clfk.cllld.cd.cllld.jeld.O",
    "N*..jk.......  .......jk..*O",
    "Kib.jk.ab.aiiiiiib.ab.jk.aiL",
    "Ild.cd.jk.cllfelld.jk.cd.clJ",
    "N......jk....jk....jk......O",
    "N.aiiiihgiib.jk.aiihgiiiib.O",
    "N.clllllllld.cd.clllllllld.O",
    "N..........................O",
    "CPPPPPPPPPPb ab aPPPPPPPPPPD",
    "           N jk O           ",
    "           N jk O           ",
    "           N jk O           ",]);
    offsets
}
