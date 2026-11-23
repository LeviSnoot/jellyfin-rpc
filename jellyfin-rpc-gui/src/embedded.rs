//! Styles and fonts embedded into the binary by `build.rs`, so the app runs
//! as a single executable without an external `assets/` directory.

include!(concat!(env!("OUT_DIR"), "/embedded.rs"));
