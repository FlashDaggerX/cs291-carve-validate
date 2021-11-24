extern crate getopts;

use getopts::Options;
use hex::FromHex;
use md5::compute;
use memmem::{Searcher, TwoWaySearcher};
use std::env::args;
use std::fs::{read, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::thread;
use std::vec::Vec;

static THREADS: usize = 8;
static EXPECTED_BYTES_MAX: usize = 200000;

fn create_file(name: String, hashes: Vec<String>, ofs: u64) {
    let mut carvefile = File::open(&name).expect("Failed to open file handle.");
    carvefile
        .seek(SeekFrom::Start(ofs))
        .expect("Couldn't seek in file thread.");

    let mut curofs = ofs;
    let mut bytes = 0;
    let mut contents = Vec::with_capacity(EXPECTED_BYTES_MAX);

    'build: while bytes <= EXPECTED_BYTES_MAX {
        curofs += 1;
        bytes = (curofs - ofs) as usize;

        Read::by_ref(&mut carvefile)
            .take(bytes as u64)
            .read_to_end(&mut contents)
            .unwrap();
        carvefile
            .seek(SeekFrom::Start(ofs))
            .expect("Couldn't seek in file thread.");

        for hash in &hashes {
            let chash = format!("{:x}", compute(&contents[..bytes-1]));
            let chash = chash.trim();

            if contents.len() <= 0 {
                println!("Comparing byte size zero, could not find suitable hash. Exiting thread at offset {}", curofs);
                break 'build;
            }

            if chash == hash.to_lowercase() {
                println!(
                    "Successfully found file matching hash {}, with offsets {}-{} (size {})",
                    hash, ofs, curofs, bytes
                );

                let mut newfile = File::create(format!("{}.carve", hash)).unwrap();
                newfile.write_all(contents.as_slice()).unwrap();
                newfile.flush().unwrap();

                break 'build;
            }
        }

        contents.clear();
    }

    println!("Exiting from carving thread at offset {}", curofs);
}

fn main() -> Result<(), std::io::Error> {
    let mut opts = Options::new();

    opts.reqopt("f", "", "The file to be carved", "FILE")
        .reqopt("h", "", "A line-delimited list of MD5 hashes", "FILE")
        .reqopt("e", "", "A line-delimited list of magic numbers", "FILE");

    let sysargs: Vec<String> = args().collect();
    let matches = opts.parse(&sysargs[1..]);
    if matches.is_err() {
        println!("{}", opts.usage(""));
        ()
    }
    let matches = matches.ok().unwrap();

    let carvename = matches.opt_str("f").expect("Option f not supplied.");
    let hashname = matches.opt_str("h").expect("Option h not supplied.");
    let extname = matches.opt_str("e").expect("Option e not supplied.");

    let hashes: Vec<String> =
        String::from_utf8(read(hashname).expect("Can't read from hash file."))
            .unwrap()
            .split("\n")
            .map(|s| s.trim().to_string())
            .filter(|s| s.len() > 0)
            .collect();
    // Temporary value stored in 'extensions' to allow borrow
    let extensions = String::from_utf8(read(extname).unwrap()).unwrap();
    let extensions: Vec<Vec<u8>> = extensions
        .split("\n")
        .map(|s| Vec::from_hex(s).expect("Invalid Hex String"))
        .filter(|s| s.len() > 0)
        .collect();

    let mut carvefile = File::open(&carvename).unwrap();
    let mut buffer: [u8; 24] = [0; 24];

    let mut threads: Vec<thread::JoinHandle<()>> = Vec::with_capacity(THREADS);

    while let Ok(bufofs) = carvefile.read(&mut buffer) {
        if bufofs <= 0 {
            break;
        }

        if threads.len() >= THREADS {
            if let Some(cur_thread) = threads.pop() {
                cur_thread.join().expect("Unable to join thread.");
            }
        }

        for ext in &extensions {
            let searcher = TwoWaySearcher::new(ext);
            if let Some(_) = searcher.search_in(&buffer) {
                let ofs = carvefile
                    .stream_position()
                    .expect("Can't get stream position.");
                let ofs = ofs - bufofs as u64;
                let name = carvename.clone();
                let hash = hashes.clone();

                println!(
                    "Found a matching extension ({:?} or {:?}) at {}",
                    hex::encode(ext),
                    String::from_utf8_lossy(ext),
                    ofs
                );

                threads.push(thread::spawn(move || create_file(name, hash, ofs)));
            }
        }

        carvefile
            .seek(SeekFrom::Current((512 - bufofs) as i64))
            .expect("Couldn't seek file from main thread.");
    }

    // Close any remaining threads
    for thread in threads {
        thread.join().expect("Can't join last thread.");
    }

    Ok(())
}
