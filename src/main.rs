extern crate getopts;

use getopts::Options;
use std::env::args;
use std::fs::{File, read};

fn main() {
    let mut opts = Options::new();

    opts
        .reqopt("f", "", "The file to be carved from", "FILE")
        .reqopt("h", "", "The file containig a line-delimited list of ", "FILE")
        .reqopt("e", "", "The file with a line-delimited list of extension numbers", "FILE");
    
    let sysargs: Vec<String> = args().collect();
    let matches = opts.parse(&sysargs[1..]);
    if matches.is_err() {
        println!("{}", opts.usage(""));
        ()
    }
    let matches = matches.ok().unwrap();

    let carvename = matches.opt_str("f").unwrap();
    let hashname = matches.opt_str("h").unwrap();
    let extname = matches.opt_str("e").unwrap();

    let carvefile = File::open(carvename).unwrap();
    let hashes: Vec<String> = String::from_utf8(read(hashname).unwrap())
            .unwrap()
            .split("\n")
            .map(|s| s.to_string())
            .collect();
    let extensions: Vec<String> = String::from_utf8(read(extname).unwrap())
            .unwrap()    
            .split("\n")
            .map(|s| s.to_string())
            .collect();
}

