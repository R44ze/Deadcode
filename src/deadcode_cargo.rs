[package]
name = "deadcode"
version = "0.1.0"
edition = "2021"
authors = ["DeadCode Team"]
description = "DeadCode - High-level systems programming language for CursedOS"

[[bin]]
name = "deadcode"
path = "src/main.rs"

[lib]
name = "deadcode_core"
path = "src/lib.rs"

[dependencies]
# CLI
clap = { version = "4.5", features = ["derive"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

[dev-dependencies]
criterion = "0.5"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1

[profile.dev]
opt-level = 0