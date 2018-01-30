extern crate regex;

use std::process::Command;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use regex::Regex;

fn get_tty() -> String {
    let output = Command::new("w")
                         .arg("-hs")
                         .output()
                         .expect("failed to execute process.");

    match String::from_utf8(output.stdout) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    }
}

fn extract_pid(tty: &str) -> (String, String) {
    let psreg = Regex::new(&format!(r"\n(\S+)\s+(\d+)\s+\S+\s+\S+\s+\S+\s+\S+\s+\?\s+\S+\s+\S+\s+\S+\s+\S+[\|\\_ ]+\S*\bsshd\b.*\n\S+\s+\S+\s+\S+\s+\S+\s+\S+\s+\S+\s+{}\s", tty))
                      .unwrap();

    // let buf = Command::new("ps")
    //                   .arg("fauwwx")
    //                   .output()
    //                   .expect("failed to execute process.");
    // let output = match String::from_utf8(buf.stdout) {
    //     Ok(v) => v,
    //     Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    // };

    let file = File::open("./sample/tty_sample.txt").unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut output = String::new();
    buf_reader.read_to_string(&mut output).unwrap();

    if !psreg.is_match(&output) {
        panic!("Unable to locate corresponding ssh session for [{}]", get_tty());
    }

    let caps = match psreg.captures(&output) {
        Some(cap) => cap,
        None => panic!("Unable to locate corresponding ssh session for [{}]", get_tty()),
    };

    let u_name  = caps.get(1).unwrap().as_str().to_string();
    let u_id    = caps.get(2).unwrap().as_str().to_string();

    (u_name, u_id)
}

fn exec_strace(pid: &str, output: &str) {
    let cmd = Command::new("strace")
                      .arg(format!("-e read -s16384 -q -x -p {} -o {}", pid, output))
                      .spawn()
                      .expect("failed to execute process.");
}

fn main() {
    let (u_name, u_id) = extract_pid("pts/0");

    println!("USER: {:?}", u_name);
    println!("PID : {:?}", &u_id);
}
