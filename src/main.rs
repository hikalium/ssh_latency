extern crate rexpect;

use rexpect::session::PtySession;
use rexpect::spawn;
use std::cmp;
use std::env;
use std::thread::sleep;
use std::time::{Duration, Instant};

fn mean(v: &[u128]) -> f32 {
    let sum = v.iter().sum::<u128>() as f32;
    let count = v.len();
    let mean = sum / count as f32;
    mean
}

fn variance(v: &[u128]) -> f32 {
    let mean = mean(v);
    let variance = v
        .iter()
        .map(|current| (*current as f32 - mean).powi(2))
        .sum::<f32>() as f32
        / (v.len() - 1) as f32;
    variance
}

fn stddev(v: &[u128]) -> f32 {
    variance(v).sqrt()
}

fn open_ssh_connection(hostname: &str) -> PtySession {
    let cmd = format!("ssh -t {}", hostname);
    let mut p = spawn(&cmd, Some(30000)).unwrap_or_else(|e| panic!("Failed to launch ssh: {}", e));
    println!("ssh opened. (Touching security key may be needed.)");
    p.exp_regex(r"\$|#").unwrap();
    println!("prompt found");
    p
}

fn ping_and_increment_seq(ssh: &mut PtySession, seq: &usize) -> Duration {
    let now = Instant::now();
    let cmd = format!("echo \"hello\" {}", seq);
    let expected = format!("hello {}", seq);
    ssh.send_line(&cmd).unwrap();
    ssh.exp_regex(&expected).unwrap();
    let elapsed = now.elapsed();
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

    let mut max = 0;
    let mut min = std::u128::MAX;
    let mut elapsed_times_ms: Vec<u128> = vec![];

    loop {
        let elapsed = ping_and_increment_seq(&mut ssh, &elapsed_times_ms.len());
        let elapsed_ms = elapsed.as_millis();
        elapsed_times_ms.push(elapsed_ms);

        let mean = mean(&elapsed_times_ms);
        let stddev = stddev(&elapsed_times_ms);
        max = cmp::max(max, elapsed_ms);
        min = cmp::min(min, elapsed_ms);
        println!(
            "{:?} mean: {}, sd: {}, min: {}, max: {}, count: {}",
            elapsed,
            mean,
            stddev,
            min,
            max,
            elapsed_times_ms.len()
        );
        sleep(Duration::new(1, 0));
    }
}

#[test]
fn test_stddev() {
    let data = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    assert_eq!(mean(&data), 5.0);
    assert_eq!(variance(&data), 11.0);
    assert_eq!(stddev(&data), 11.0_f32.sqrt());
}
