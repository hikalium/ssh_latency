extern crate rexpect;

use rexpect::session::PtySession;
use rexpect::spawn;
use std::cmp;
use std::env;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn open_ssh_connection(hostname: &str) -> PtySession {
    let cmd = format!("ssh {}", hostname);
    let mut p = spawn(&cmd, Some(30000))
        .unwrap_or_else(|e| panic!("Failed to launch ssh: {}", e));
    println!("ssh opened. (Touching security key may be needed.)");
    p.exp_regex(r"\$").unwrap();
    println!("prompt found");
    p
}

fn ping_and_increment_seq(ssh: &mut PtySession, seq: &mut usize) -> Duration {
    let now = Instant::now();
    let cmd = format!("echo \"hello\" {}", seq);
    let expected = format!("hello {}", seq);
    ssh.send_line(&cmd).unwrap();
    ssh.exp_regex(&expected).unwrap();
    let elapsed = now.elapsed();
    *seq += 1;
    elapsed
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: ssh_latency <host>");
        return;
    }
    let host = &args[1];
    let mut ssh = open_ssh_connection(&host);
    let mut seq = 0;

    let mut avg = 0;
    let mut max = 0;
    let mut min = std::u128::MAX;

    loop {
        let elapsed = ping_and_increment_seq(&mut ssh, &mut seq);
        let elapsed_ms = elapsed.as_millis();
        avg = (avg * (seq - 1) as u128 + elapsed_ms) / seq as u128;
        max = cmp::max(max, elapsed_ms);
        min = cmp::min(min, elapsed_ms);
        println!(
            "{:?} avg: {}, min: {}, max: {}, count: {}",
            elapsed, avg, min, max, seq
        );
        sleep(Duration::new(1, 0));
    }
}
