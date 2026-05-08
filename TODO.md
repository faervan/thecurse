# Bugs
- [x] Goblins can end up in Idle state even if they have a target
- [ ] Creatures can potentially hit themselves
- [ ] Avian panic in `attack_changes`
    ```
    thread 'main' (133624) panicked at /home/stk/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/avian3d-0.5.0/src/collision/collider/mod.rs:512:9:
    assertion failed: b.min.cmple(b.max).all()
    note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
    Encountered a panic when applying buffers for system `thecurse_core::character_controller::actions::attack::attack_changes`!
    Encountered a panic in system `bevy_ecs::apply_deferred`!
    Encountered a panic in system `bevy_app::main_schedule::Main::run_main`!
    ```
    - Appears to happen only or at least deterministically when the player is invisible
- [ ] Goblins Idle animation is interrupted when they retry path finding

# Features
- [ ] implement enforce LoS in creature target finding
