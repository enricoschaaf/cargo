use crate::prelude::*;
use cargo_test_support::project;
use cargo_test_support::registry::Package;

#[cargo_test]
fn shared_graph() {
    const LAYERS: usize = 18;

    Package::new("shared", "0.1.0")
        .feature("a", &[])
        .file("src/lib.rs", "pub fn fun() {}")
        .publish();
    for layer in (0..=LAYERS).rev() {
        for side in ["left", "right"] {
            let name = format!("layer_{layer:02}_{side}");
            let mut package = Package::new(&name, "0.1.0");
            if layer == LAYERS {
                package.dep("shared", "0.1.0");
            } else {
                package.dep(&format!("layer_{:02}_left", layer + 1), "0.1.0");
                package.dep(&format!("layer_{:02}_right", layer + 1), "0.1.0");
            }
            package.file("src/lib.rs", "pub fn fun() {}").publish();
        }
    }

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.1.0"
            authors = []
            edition = "2018"

            [dependencies]
            layer_00_left = "0.1.0"
            shared = { version = "0.1.0", features = ["a"] }

            [lints.cargo]
            default = { level = "allow", priority = -1 }
            unused_dependencies = "warn"
        "#,
        )
        .file(
            "src/main.rs",
            r#"
            fn main() { layer_00_left::fun(); }
            "#,
        )
        .build();

    p.cargo("check -Zcargo-lints")
        .masquerade_as_nightly_cargo(&["cargo-lints"])
        .with_stderr_does_not_contain("[WARNING] unused dependency")
        .run();
}
