use compressed_set::CompressedSequence;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn make_set() -> CompressedSequence {
    let mut comp_seq = CompressedSequence::new(10);

    let mut exp = vec![];

    for (pos, i) in (0..=200_000).step_by(10).enumerate() {
        comp_seq.push(i);
        exp.push(i);

        if pos % 10 == 0 {
            comp_seq.push(i + 1);
            exp.push(i + 1);
        }
    }

    comp_seq
}

fn index_item_decode(c: &mut Criterion) {
    c.bench_function("get", |b| {
        let set = make_set();
        let len = set.len();
        b.iter(|| {
            for i in (0..len).step_by(333) {
                let _ = set.get(black_box(i));
            }
        });
    });

    c.bench_function("contains", |b| {
        let set = make_set();
        let len = set.len();
        b.iter(|| {
            for i in (10000..len).step_by(333) {
                let _ = set.contains(black_box(i as u32));
            }
        });
    });

    c.bench_function("bin_search", |b| {
        let set = make_set();
        b.iter(|| {
            let _ = set.has_bin_search(black_box(333 as u32));
        });
    });

    c.bench_function("bin_search_indexed", |b| {
        let mut set = make_set();
        set.update_index(50.0);
        b.iter(|| {
            let _ = set.has_bin_search(black_box(333 as u32));
        });
    });
}

criterion_group!(benches, index_item_decode);
criterion_main!(benches);
