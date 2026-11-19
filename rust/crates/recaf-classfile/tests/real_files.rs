//! Parses real class files shipped in the repository's test fixtures.
//!
//! The fixtures come from Recaf's own test samples: `obf-sample-reference.jar`
//! (a normal javac build) and `obf-sample-zkm-26.jar` (the same sample after
//! Zelix KlassMaster obfuscation). This guards the parser against real-world
//! layouts beyond the hand-crafted bytes in `crafted.rs`.

use recaf_classfile::*;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_class_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("fixtures directory must exist") {
        let path = entry.expect("fixture entry readable").path();
        if path.is_dir() {
            collect_class_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("class") {
            out.push(path);
        }
    }
}

#[test]
fn parses_all_real_fixture_class_files() {
    let mut files = Vec::new();
    collect_class_files(Path::new("tests/fixtures"), &mut files);
    assert!(
        files.len() >= 4,
        "expected at least 4 committed fixtures, found {}",
        files.len()
    );

    for path in files {
        let bytes = fs::read(&path).expect("fixture readable");
        let class = ClassFile::parse(&bytes)
            .unwrap_or_else(|error| panic!("{} failed to parse: {error}", path.display()));

        // Generic invariants that must hold for every valid class file.
        assert!(
            class.major_version >= MIN_MAJOR_VERSION,
            "{}",
            path.display()
        );
        assert!(class.constant_pool.valid_count() > 0, "{}", path.display());
        assert!(!class.this_class.is_empty(), "{}", path.display());
        let _ = class.java_version();

        for field in &class.fields {
            assert!(!field.name.is_empty(), "{}", path.display());
            assert!(!field.descriptor.is_empty(), "{}", path.display());
        }
        for method in &class.methods {
            assert!(!method.name.is_empty(), "{}", path.display());
            assert!(!method.descriptor.is_empty(), "{}", path.display());
        }
    }
}

#[test]
fn reference_fixture_has_expected_members() {
    let bytes = fs::read("tests/fixtures/Fields.class").expect("fixture readable");
    let class = ClassFile::parse(&bytes).expect("reference fixture must parse");

    assert!(
        class.this_class.starts_with("sample/"),
        "this_class = {}",
        class.this_class
    );
    assert!(!class.fields.is_empty());
    assert!(!class.methods.is_empty());
    assert!(class
        .methods
        .iter()
        .any(|m| m.attribute_names.iter().any(|a| a == "Code")));
}

#[test]
fn obfuscated_fixture_still_parses() {
    // The ZKM-obfuscated build of the same class must parse as cleanly as
    // the reference build, whatever names the obfuscator produced.
    let bytes = fs::read("tests/fixtures/zkm26/Fields.class").expect("fixture readable");
    let class = ClassFile::parse(&bytes).expect("obfuscated fixture must parse");

    assert!(class.major_version >= MIN_MAJOR_VERSION);
    assert!(!class.this_class.is_empty());
}
