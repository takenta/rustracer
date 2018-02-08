extern crate regex;
extern crate logwatcher;

use std::process::Command;
use std::env;
use regex::Regex;
use logwatcher::LogWatcher;

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

    let buf = Command::new("ps")
                      .arg("fauwwx")
                      .output()
                      .expect("failed to execute process.");
    let output = match String::from_utf8(buf.stdout) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

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
    let mut cmd = Command::new("strace")
        .arg("-e")
        .arg("read")
        .arg("-s16384")
        .arg("-q")              
        .arg("-x")              
        .arg("-p")              
        .arg(pid)
        .arg("-o")              
        .arg(output)              
        .spawn()
        .expect("failed to execute process.");

    let readreg = Regex::new("(read)\\(\\d+, \"(.*)\")").unwrap();

    let mut watcher = LogWatcher::register(output.to_string()).unwrap();

    watcher.watch(|line: String| {
        let caps = match readreg.captures(&line) {
            Some(cap) => cap,
            None => panic!("Unable to locate corresponding ssh session for {}", get_tty()),
        };
        let ichar = caps.get(2).unwrap().as_str().to_string();
        println!("{}", ichar);
    });
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (u_name, u_id) = extract_pid(&args[1]);

    println!("USER: {:?}", u_name);
    println!("PID : {:?}", &u_id);

    exec_strace(&u_id, "sample/output.txt")
}
