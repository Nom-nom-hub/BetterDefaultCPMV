use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_file(path: &PathBuf, size: u64) {
    let mut file = File::create(path).unwrap();
    let chunk = vec![42u8; 1024 * 1024]; // 1 MB chunks

    let num_chunks = size / (1024 * 1024);
    for _ in 0..num_chunks {
        file.write_all(&chunk).unwrap();
    }

    // Write remaining bytes
    let remainder = size % (1024 * 1024);
    if remainder > 0 {
        file.write_all(&vec![42u8; remainder as usize]).unwrap();
    }
}

fn bench_small_file_copy(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let src_path = temp_dir.path().join("source_small.bin");
    let dst_path = temp_dir.path().join("dest_small.bin");

    create_test_file(&src_path, 10 * 1024 * 1024); // 10 MB

    c.bench_function("copy_10mb_file", |b| {
        b.iter(|| {
            let _ = fs::copy(black_box(&src_path), black_box(&dst_path));
            let _ = fs::remove_file(&dst_path);
        });
    });
}

fn bench_medium_file_copy(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let src_path = temp_dir.path().join("source_medium.bin");
    let dst_path = temp_dir.path().join("dest_medium.bin");

    create_test_file(&src_path, 100 * 1024 * 1024); // 100 MB

    c.bench_function("copy_100mb_file", |b| {
        b.iter(|| {
            let _ = fs::copy(black_box(&src_path), black_box(&dst_path));
            let _ = fs::remove_file(&dst_path);
        });
    });
}

fn bench_large_file_copy(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let src_path = temp_dir.path().join("source_large.bin");
    let dst_path = temp_dir.path().join("dest_large.bin");

    create_test_file(&src_path, 500 * 1024 * 1024); // 500 MB

    c.bench_function("copy_500mb_file", |b| {
        b.iter(|| {
            let _ = fs::copy(black_box(&src_path), black_box(&dst_path));
            let _ = fs::remove_file(&dst_path);
        });
    });
}

fn bench_many_small_files(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("source_dir");
    let dst_dir = temp_dir.path().join("dest_dir");

    fs::create_dir(&src_dir).unwrap();

    // Create 100 small files (100 KB each)
    for i in 0..100 {
        let file_path = src_dir.join(format!("file_{:03}.bin", i));
        create_test_file(&PathBuf::from(&file_path), 100 * 1024);
    }

    c.bench_function("copy_100_small_files", |b| {
        b.iter(|| {
            if dst_dir.exists() {
                let _ = fs::remove_dir_all(&dst_dir);
            }
            fs::create_dir(&dst_dir).unwrap();

            for entry in fs::read_dir(&src_dir).unwrap() {
                let entry = entry.unwrap();
                let src = entry.path();
                let dst = dst_dir.join(entry.file_name());
                let _ = fs::copy(&src, &dst);
            }
        });
    });
}

fn bench_directory_structure(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("source_complex");
    let dst_dir = temp_dir.path().join("dest_complex");

    // Create nested directory structure
    fs::create_dir_all(src_dir.join("dir1/dir2/dir3")).unwrap();
    fs::create_dir_all(src_dir.join("dir4/dir5")).unwrap();

    // Add files at various levels
    for i in 0..5 {
        create_test_file(&PathBuf::from(src_dir.join(format!("file_{}.bin", i))), 1024 * 1024);
        create_test_file(&PathBuf::from(src_dir.join(format!("dir1/file_{}.bin", i))), 1024 * 1024);
        create_test_file(&PathBuf::from(src_dir.join(format!("dir1/dir2/file_{}.bin", i))), 1024 * 1024);
    }

    c.bench_function("copy_nested_directory_structure", |b| {
        b.iter(|| {
            if dst_dir.exists() {
                let _ = fs::remove_dir_all(&dst_dir);
            }

            fn copy_dir(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
                fs::create_dir(dst)?;
                for entry in fs::read_dir(src)? {
                    let entry = entry?;
                    let path = entry.path();
                    let file_name = entry.file_name();
                    let dest_path = dst.join(&file_name);

                    if path.is_dir() {
                        copy_dir(&path, &dest_path)?;
                    } else {
                        fs::copy(&path, &dest_path)?;
                    }
                }
                Ok(())
            }

            let _ = copy_dir(&src_dir, &dst_dir);
        });
    });
}

criterion_group!(
    benches,
    bench_small_file_copy,
    bench_medium_file_copy,
    bench_large_file_copy,
    bench_many_small_files,
    bench_directory_structure
);

criterion_main!(benches);
