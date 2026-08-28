# Project Workflow

## Guiding Principles

1. **The Plan is the Source of Truth:** All work must be tracked in `plan.md`
2. **The Tech Stack is Deliberate:** Changes to the tech stack must be documented in `tech-stack.md` *before* implementation
3. **Test-Driven Development:** Write unit tests before implementing functionality
4. **High Code Quality:** Keep code compiler-warning free with tests passing via `cargo test`
5. **Continuous Verification:** Verify visual and gameplay fidelity against original game assets and behavior

## Task Workflow

1. **Select Task:** Choose the next available task from `plan.md` in sequential order.
2. **Mark In Progress:** Edit `plan.md` and change the task from `[ ]` to `[~]`.
3. **Test First (TDD):** Where applicable (e.g. parsers, math routines, VM opcode execution), write unit tests before implementing code.
4. **Implement & Pass:** Implement the necessary logic and confirm `cargo test` and `cargo check` pass cleanly.
5. **Phase Checkpoint & Verification:** Manually test in the running Bevy window (`cargo run`) to verify visual, audio, and physics behavior.
6. **Mark Complete:** Update `plan.md` to `[x]` and commit progress.
