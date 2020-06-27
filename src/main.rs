//  --                                                          ; {{{1
//
//  File        : src/main.rs
//  Maintainer  : Felix C. Stegerman <flx@obfusk.net>
//  Date        : 2020-06-27
//
//  Copyright   : Copyright (C) 2020  Felix C. Stegerman
//  Version     : v0.1.2
//  License     : GPLv3+
//
//  --                                                          ; }}}1

const VERSION: &str = "0.1.2";

use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::iter;
use std::process;

use structopt::StructOpt;

macro_rules! oops {
  ($msg:expr)   => {{ oops($msg); panic!() }};
  ($($arg:tt)*) => {{ oops(format!($($arg)*)); panic!() }}
}

fn oops(msg: String) { eprintln!("{}", msg); process::exit(1) }

#[derive(Clone, Copy, Debug)]
struct Rgb(u8, u8, u8);

fn rgbto8(c: &Rgb) -> u8 {
  (16 + (c.0 as u16)*6/256*36 + (c.1 as u16)*6/256*6 + (c.2 as u16)*6/256) as u8
}

const FLAGS: &str = "\
  pride, agender, aromantic, asexual, bisexual, genderfluid,
  genderqueer, lesbian, nonbinary, pansexual, polysexual, transgender";

// https://en.wikipedia.org/wiki/Pride_flag#Gallery
fn flag2colours(flag: &str) -> Vec<Rgb> {                     //  {{{1
  let f = match flag {
    "lgbt" => "pride", "aro" => "aromantic", "ace" => "asexual",
    "bi" => "bisexual", "enby" => "nonbinary", "nb" => "nonbinary",
    "pan" => "pansexual", "trans" => "transgender", _ => flag
  };
  let cs = match f {
    "pride"       => vec![Rgb(228, 3, 3), Rgb(255, 140, 0), Rgb(255, 237, 0),
                          Rgb(0, 128, 38), Rgb(0, 77, 255), Rgb(117, 7, 135)],
    "agender"     => vec![Rgb(0, 0, 0), Rgb(185, 185, 185), Rgb(255, 255, 255),
                          Rgb(184, 244, 131),
                          Rgb(255, 255, 255), Rgb(185, 185, 185), Rgb(0, 0, 0)],
    "aromantic"   => vec![Rgb(61, 165, 66), Rgb(167, 211, 121),
                          Rgb(255, 255, 255), Rgb(169, 169, 169), Rgb(0, 0, 0)],
    "asexual"     => vec![Rgb(0, 0, 0), Rgb(163, 163, 163),
                          Rgb(255, 255, 255), Rgb(128, 0, 128)],
    "bisexual"    => vec![Rgb(214, 2, 112), Rgb(214, 2, 112), Rgb(155, 79, 150),
                          Rgb(0, 56, 168), Rgb(0, 56, 168)],
    "genderfluid" => vec![Rgb(255, 117, 162), Rgb(255, 255, 255),
                          Rgb(190, 24, 214), Rgb(0, 0, 0), Rgb(51, 62, 189)],
    "genderqueer" => vec![Rgb(181, 126, 220), Rgb(255, 255, 255), Rgb(74, 129, 35)],
    "lesbian"     => vec![Rgb(213, 45, 0), Rgb(255, 154, 86), Rgb(255, 255, 255),
                          Rgb(211, 98, 164), Rgb(163, 2, 98)],
    "nonbinary"   => vec![Rgb(255, 244, 48), Rgb(255, 255, 255),
                          Rgb(156, 89, 209), Rgb(0, 0, 0)],
    "pansexual"   => vec![Rgb(255, 33, 140), Rgb(255, 216, 0), Rgb(33, 177, 255)],
    "polysexual"  => vec![Rgb(246, 28, 185), Rgb(7, 213, 105), Rgb(28, 146, 246)],
    "transgender" => vec![Rgb(91, 206, 250), Rgb(245, 169, 184), Rgb(255, 255, 255),
                          Rgb(245, 169, 184), Rgb(91, 206, 250)],
    _ => oops!("unknown flag: {}", flag)
  };
  if cs.len() > 3 { cs } else {
    cs.iter().flat_map(|&c| iter::repeat(c).take(2)).collect()
  }
}                                                             //  }}}1

const BLACK: Rgb = Rgb(0, 0, 0);
const WHITE: Rgb = Rgb(255, 255, 255);

fn colours(flags: Vec<String>) -> Vec<Rgb> {
  flags.iter().flat_map(|f| flag2colours(f)).collect()
}

fn detect_truecolor() -> bool {
  let ct = env::var("COLORTERM").unwrap_or("".to_string());
  ct.contains("truecolor") || ct.contains("24bit")
}

fn with_colour(bg: bool, tc: bool, c: &Rgb, s: &str, clear: bool) -> String {
  let (sc, rc)  = if bg { (48, 49) } else { (38, 39) };
  let cl        = if bg && clear { "\x1b[2K" } else { "" };
  let code      = if tc { format!("2;{};{};{}", c.0, c.1, c.2) }
                   else { format!("5;{}", rgbto8(c)) };
  format!("\x1b[{};{}m{}{}\x1b[{};m", sc, code, cl, s, rc)
}

fn colour(bg: bool, tc: bool, li: bool, c: &Rgb,
          s: &str, clear: bool) -> String {
  let da = if li { bg } else { !bg };
  let t = if (da && brightness(c) < 0.20) || (!da && brightness(c) > 0.75) {
    with_colour(!bg, tc, if da { &WHITE } else { &BLACK }, s, clear)
  } else { s.to_string() };
  with_colour(bg, tc, c, t.as_ref(), clear)
}

fn brightness(c: &Rgb) -> f32 {
  (((c.0 as u32) * 299 + (c.1 as u32) * 587 + (c.2 as u32) * 114) / 1000)
    as f32 / 255f32
}

const HELP_TEMPLATE: &str = "\
Usage: {usage}

  proudcat-rust - cat + rainbow

  Flags: {after-help}.

  Aliases: lgbt, aro, ace, bi, enby, nb, pan, trans.

Options:
{unified}";

#[derive(Debug, StructOpt)]
#[structopt(name = "proudcat-rust", version = VERSION,
  after_help = FLAGS, template = HELP_TEMPLATE)]
struct Cli {                                                  //  {{{1
  #[structopt(name = "flag", short = "f", long = "flag",
    number_of_values = 1, display_order = 1,
    help = "Choose which flags to use; default: pride")]
  flags: Vec<String>,
  #[structopt(short, long = "background", display_order = 2,
    help = "Set background colour (instead of foreground)")]
  bg: bool,
  #[structopt(short = "t", long = "truecolor", display_order = 3,
    help = "Explicitly enable 24-bit colours; default: autodetect")]
  tc: bool,
  #[structopt(short = "T", long = "no-truecolor", display_order = 4,
    conflicts_with = "tc", help = "Explicitly disable 24-bit colours")]
  no_tc: bool,
  #[structopt(long, display_order = 5, help = "Light terminal")]
  light: bool,
  #[structopt(long, display_order = 6, help = "Demonstrate flags")]
  demo: bool,
  #[structopt(name = "FILES", default_value = "-")]
  files: Vec<String>
}                                                             //  }}}1

fn parse_args() -> Cli {
  let mut cli = Cli::from_args();
  cli.tc      = if cli.no_tc { false } else if cli.tc { true }
                else { detect_truecolor() };
  if cli.flags.is_empty() { cli.flags = vec!["pride".to_string()] }
  cli
}

// TODO
fn stdout_isatty() -> bool {
  if !cfg!(unix) { true } else {
    unsafe { libc::isatty(libc::STDOUT_FILENO) != 0 }
  }
}

fn main() {
  let (tty, stdin, cli) = (stdout_isatty(), io::stdin(), parse_args());
  if cli.demo {
    for flag in FLAGS.split_whitespace().map(|x| x.trim_end_matches(",")) {
      println!("┌{}┐", "─".repeat(flag.len()));
      for c in flag2colours(flag) {
        let s = colour(cli.bg, cli.tc, cli.light, &c, flag, false);
        println!("│{}│", s)
      }
      println!("└{}┘", "─".repeat(flag.len()))
    }
  } else {
    let clrs    = colours(cli.flags.iter().flat_map(|a|
                    a.split(",").map(|a| a.to_string())
                  ).collect());
    let mut it  = clrs.iter().cycle();
    for file in cli.files {
      let mut line = String::new();
      let mut bufr = if &file == "-" {
        Box::new(stdin.lock()) as Box<dyn BufRead>
      } else {
        Box::new(io::BufReader::new(
          File::open(&file).unwrap_or_else(|e|
            oops!("Could not open file: {}: {}", &file, e.to_string())
        )))
      };
      while bufr.read_line(&mut line).unwrap() != 0 {
        let sline = line.trim();
        if sline.is_empty() && !(cli.bg && tty) {
          print!("{}", line)
        } else {
          let s = colour(cli.bg, cli.tc, cli.light, it.next().unwrap(),
                         sline, true);
          let i = sline.chars().next().and_then(|c| line.find(c))
                                      .unwrap_or(0);
          print!("{}{}{}", line[..i].to_string(), s,
                 line[i+sline.len()..].to_string())
        }
        line.clear()
      }
    }
  }
}

// vim: set tw=70 sw=2 sts=2 et fdm=marker :
