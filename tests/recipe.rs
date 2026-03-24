use std::path::PathBuf;

use rstest::rstest;

use fujinx::Recipe;

#[rstest]
fn roundtrip(#[files("tests/recipes/*.yaml")] path: PathBuf) {
    let yaml = std::fs::read_to_string(&path).unwrap();
    let recipe: Recipe = serde_yaml::from_str(&yaml).unwrap();
    let re_encoded = serde_yaml::to_string(&recipe).unwrap();
    assert_eq!(yaml, re_encoded, "round-trip failed for {}", path.display());
}
