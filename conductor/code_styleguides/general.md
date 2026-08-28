# General Code Style Guide

1. **Clarity & Readability**: Prefer clear, descriptive variable and function names over obscure abbreviations.
2. **Deterministic State**: Avoid hidden global state; encapsulate state within Bevy ECS Resources and Components.
3. **Robust Error Handling**: Avoid unhandled `panic!`, `unwrap()`, or `expect()` in runtime code; return `Result` or `Option` where failure is possible (especially in file parsing).
4. **Documentation**: Keep public APIs and data structure mappings clearly documented with links/comments explaining original Build engine field meanings.
