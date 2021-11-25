use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use adsb_deku::deku::prelude::*;
use adsb_deku::Frame;
use criterion::{criterion_group, criterion_main, Criterion};

fn lax_message() {
    // Read from test file and assert display implemented and non panic decode
    let file = File::open("tests/lax-messages.txt").unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        let len = line.chars().count();
        let hex = &mut line.to_string()[1..len - 1].to_string();
        let bytes = hex::decode(&hex).unwrap();
        // test non panic decode
        let frame = Frame::from_bytes((&bytes, 0)).unwrap().1;
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("lax_messsages", |b| b.iter(|| lax_message()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
