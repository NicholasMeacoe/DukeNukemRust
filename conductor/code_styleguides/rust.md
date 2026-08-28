# Rust Code Style Guide

1. **Formatting**: Always adhere to `cargo fmt` standard style.
2. **Linting**: Keep code clean of compiler warnings and pass `cargo clippy`.
3. **Idiomatic Rust**:
   - Prefer iterator combinators over manual indexing where suitable.
   - Use strongly-typed newtypes or enums for state rather than magic numbers.
   - Keep unsafe code strictly isolated (if any).
4. **Bevy ECS Conventions**:
   - Organize game features into dedicated Bevy `Plugin` structs.
   - Separate Systems by responsibility (Startup, Update, FixedUpdate).
   - Use Query filters (`With<T>`, `Without<T>`, `Changed<T>`) to minimize query iteration overhead.
