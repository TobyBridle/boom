use boom_core::boom::parse_bangs::parse_bang_indexes;

fn main() {
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
fn bench_bang(bang: &str) {
    #[allow(unused_must_use)]
    parse_bang_indexes(bang);
}
