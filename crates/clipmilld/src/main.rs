//! The ClipMill daemon. Real daemon skeleton lands in workstream W2; this
//! binary exists so the workspace, CI, and kill-drill wiring have a target
//! from day one.

fn main() {
    println!(
        "clipmilld {} (pre-alpha; daemon lands in W2)",
        env!("CARGO_PKG_VERSION")
    );
}
