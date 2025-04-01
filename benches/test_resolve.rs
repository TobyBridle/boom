use std::{collections::HashMap, thread::sleep, time::Duration};

use boom::{
    boom::{parse_bangs::parse_bang_file, resolver::resolve},
    cache::{init_list, insert_bang},
};

fn main() {
    let bangs = parse_bang_file(None).unwrap();
    bangs.iter().enumerate().for_each(|(idx, bang)| {
        insert_bang(bang.trigger.clone(), idx).unwrap();
    });
    init_list(bangs, false).unwrap();

    divan::main();
}

const STRINGS: [&str; 9] = [
    "!yt this is a test",
    "please work this is a test pelase work",
    "yt",
    "i was typing and !d forgot i wanted dictionary",
    "how about we want a definitoin and realise we're kinky !urban",
    "dontforgetthatthis!01net doesn't work",
    "thid doesn't work!either right",
    "!01net search for this on 01net",
    "! test !gh",
];

#[divan::bench(args = STRINGS, sample_count = 10_000)]
fn bench_resolve(query: &str) {
    resolve(query);
}
