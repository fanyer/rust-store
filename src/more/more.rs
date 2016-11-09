#![crate_name = "uu_more"]

/*
 * This file is part of the uutils coreutils package.
 *
 * (c) Martin Kysel <code@martinkysel.com>
 *
 * For the full copyright and license information, please view the LICENSE file
 * that was distributed with this source code.
 */

extern crate getopts;

#[macro_use]
extern crate uucore;

use getopts::Options;
use std::io::{stdout, Write, Read};
use std::fs::File;

extern crate nix;
use nix::sys::termios;

#[derive(Clone, Eq, PartialEq)]
pub enum Mode {
    More,
    Help,
    Version,
}

static NAME: &'static str = "more";
static VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub fn uumain(args: Vec<String>) -> i32 {
    let mut opts = Options::new();

    opts.optflag("h", "help", "display this help and exit");
    opts.optflag("v", "version", "output version information and exit");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            show_error!("{}", e);
            panic!()
        },
    };
    let usage = opts.usage("more TARGET.");
    let mode = if matches.opt_present("version") {
        Mode::Version
    } else if matches.opt_present("help") {
        Mode::Help
    } else {
        Mode::More
    };

    match mode {
        Mode::More    => more(matches),
        Mode::Help    => help(&usage),
        Mode::Version => version(),
    }

    0
}

fn version() {
    println!("{} {}", NAME, VERSION);
}

fn help(usage: &str) {
    let msg = format!("{0} {1}\n\n\
                       Usage: {0} TARGET\n  \
                       \n\
                       {2}", NAME, VERSION, usage);
    println!("{}", msg);
}

fn more(matches: getopts::Matches) {
    let files = matches.free;
    let mut f = File::open(files.first().unwrap()).unwrap();
    let mut buffer = [0; 1024];

    let mut term = termios::tcgetattr(0).unwrap();
    // Unset canonical mode, so we get characters immediately
    term.c_lflag.remove(termios::ICANON);
    // Disable local echo
    term.c_lflag.remove(termios::ECHO);
    termios::tcsetattr(0, termios::TCSADRAIN, &term).unwrap();

    let mut end = false;
    while let Ok(sz) = f.read(&mut buffer) {
        if sz == 0 { break; }
        stdout().write(&buffer[0..sz]).unwrap();
        for byte in std::io::stdin().bytes() {
            match byte.unwrap() {
                b' ' => break,
                b'q' | 27 => {
                    end = true;
                    break;
                },
                _ => ()
            }
        }

        if end { break;}
    }

    term.c_lflag.insert(termios::ICANON);
    term.c_lflag.insert(termios::ECHO);
    termios::tcsetattr(0, termios::TCSADRAIN, &term).unwrap();
    println!("");
}
