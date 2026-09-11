# Contributing to grass.ai

## Naming Conventions

### Structures and Enums
All public structures and enums MUST be prefixed with `Grass`:

```rust
// GOOD
pub struct GrassAgentConfig { ... }
pub enum GrassTaskStatus { ... }

// BAD
pub struct AgentConfig { ... }
pub enum TaskStatus { ... }
```

### Traits
All public traits MUST be prefixed with `Grass`:

```rust
// GOOD
pub trait GrassAgent: Send + Sync { ... }

// BAD
pub trait Agent: Send + Sync { ... }
```

### Exceptions
- Private/internal structs within a module do not need the prefix.
- Enums used only as variants of a `Grass`-prefixed enum can omit it.
- Function names use `snake_case` without the prefix.

## File Organization

Each crate follows this structure:

```
grass-<name>/
├── Cargo.toml
├── src/
│   ├── lib.rs          # public re-exports
│   ├── <module>.rs     # module implementations
│   └── <module>/
│       ├── mod.rs      # sub-module declarations
│       └── <impl>.rs   # sub-module implementations
├── tests/              # integration tests
│   └── <feature>_test.rs
└── examples/           # runnable examples
    └── <feature>_example.rs
```

## Code Style

- Max line width: 100 characters
- 4-space indentation (no tabs)
- All public items require `///` doc comments
- Use `thiserror` for error variants
- Use `async_trait` for async trait methods
- Prefer `Result<T, GrassError>` over `Option<T>` for error paths
- Use `tracing` macros for logging (`tracing::info!`, etc.)

## Testing

- Unit tests go in `tests/` subdirectory of each crate
- Integration tests go in the workspace `tests/` directory
- Each test file should start with `#[cfg(test)]` module
- Examples in `examples/` directory should be runnable with `cargo run --example <name>`

## Commit Messages

Use conventional commits:
- `feat(grass-llm): add groq provider`
- `fix(grass-core): resolve agent lifecycle bug`
- `docs(grass-ipc): update architecture`
- `test(grass-sandbox): add docker backend tests`
