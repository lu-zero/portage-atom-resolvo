use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use portage_atom::{Cpn, Cpv, Dep};
use portage_atom_resolvo::{InMemoryRepository, PackageDeps, PackageMetadata};

fn bench_cpn_parsing(c: &mut Criterion) {
    let inputs = [
        "dev-lang/rust",
        "app-editors/vim",
        "media-video/ffmpeg",
        "sys-kernel/gentoo-sources",
        "dev-python/numpy",
    ];

    c.bench_function("cpn_parse", |b| {
        b.iter(|| {
            for input in &inputs {
                black_box(Cpn::parse(input).unwrap());
            }
        })
    });
}

fn bench_cpv_parsing(c: &mut Criterion) {
    let inputs = [
        "dev-lang/rust-1.75.0",
        "app-editors/vim-9.0.2167",
        "media-video/ffmpeg-6.1",
        "sys-kernel/gentoo-sources-6.6.13",
        "dev-python/numpy-1.26.3",
    ];

    c.bench_function("cpv_parse", |b| {
        b.iter(|| {
            for input in &inputs {
                black_box(Cpv::parse(input).unwrap());
            }
        })
    });
}

fn bench_dep_parsing(c: &mut Criterion) {
    let inputs = [
        "dev-lang/rust",
        ">=dev-lang/rust-1.75.0",
        ">=dev-lang/rust-1.75.0:0",
        ">=dev-lang/rust-1.75.0:0[llvm_targets_AMDGPU]",
        "!!>=dev-lang/rust-1.75.0:0/1.75[llvm_targets_AMDGPU,-debug]::gentoo",
    ];

    c.bench_function("dep_parse", |b| {
        b.iter(|| {
            for input in &inputs {
                black_box(Dep::parse(input).unwrap());
            }
        })
    });
}

fn bench_cpn_comparison(c: &mut Criterion) {
    let cpns: Vec<Cpn> = (0..1000)
        .map(|i| Cpn::new(format!("cat-{}", i / 100), format!("pkg-{}", i)))
        .collect();

    c.bench_function("cpn_compare", |b| {
        b.iter(|| {
            for i in 0..cpns.len() - 1 {
                black_box(cpns[i] == cpns[i + 1]);
            }
        })
    });
}

fn bench_repository_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("repository");
    group.throughput(Throughput::Elements(1000));

    group.bench_function("add_1000_packages", |b| {
        b.iter(|| {
            let mut repo = InMemoryRepository::new();
            for i in 0..1000 {
                let cpv = Cpv::parse(&format!("cat/pkg-{i}-1.0")).unwrap();
                let meta = PackageMetadata {
                    cpv,
                    slot: None,
                    subslot: None,
                    iuse: vec![],
                    use_flags: Default::default(),
                    repo: None,
                    dependencies: PackageDeps::default(),
                };
                repo.add(meta);
            }
            black_box(repo);
        })
    });

    group.finish();
}

fn bench_string_alloc(c: &mut Criterion) {
    c.bench_function("create_same_string_1000x", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(Cpn::new("dev-lang".to_string(), "rust".to_string()));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_cpn_parsing,
    bench_cpv_parsing,
    bench_dep_parsing,
    bench_cpn_comparison,
    bench_repository_add,
    bench_string_alloc,
);

criterion_main!(benches);
