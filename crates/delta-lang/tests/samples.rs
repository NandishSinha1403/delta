//! Every Pad sample (`samples/*.delta`) runs without diagnostics and passes all its tests.

#[test]
fn every_sample_runs_and_passes() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../samples");
    let mut n = 0;
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "delta") {
            continue;
        }
        let src = std::fs::read_to_string(&path).unwrap_or_default();
        let out = std::thread::Builder::new()
            .stack_size(256 * 1024 * 1024)
            .spawn(move || delta_lang::run(&src))
            .and_then(|h| h.join().map_err(|_| std::io::Error::other("engine thread panicked")))
            .unwrap_or(serde_json::Value::Null);
        assert_eq!(out["ok"], true, "{}: {}", path.display(), out["diagnostics"]);
        assert_eq!(out["diagnostics"].as_array().map(|d| d.len()), Some(0), "{}", path.display());
        for o in out["outputs"].as_array().into_iter().flatten() {
            assert!(o["kind"] != "test" || o["pass"] == true, "{}: {o}", path.display());
        }
        // The Pad's view of the same program builds too.
        let src = std::fs::read_to_string(&path).unwrap_or_default();
        let doc = std::thread::Builder::new()
            .stack_size(256 * 1024 * 1024)
            .spawn(move || delta_lang::pad::document(&src))
            .and_then(|h| h.join().map_err(|_| std::io::Error::other("engine thread panicked")))
            .unwrap_or(serde_json::Value::Null);
        assert!(doc["objects"].is_array(), "{}", path.display());
        n += 1;
    }
    assert!(n >= 20, "expected at least 20 samples, found {n}");
}
