# Build artifacts during migration

Each author/reviewer uses an exclusive worktree and target directory. Development and test profiles default to `debug = false` and `incremental = false` to keep repeated builds small. To opt in for a debugging session:

```sh
CARGO_PROFILE_DEV_DEBUG=2 CARGO_PROFILE_TEST_DEBUG=2 CARGO_INCREMENTAL=1 cargo test
```

When a validation run or merged PR is finished, remove its disposable Cargo target once no build, test, benchmark or library process is using it. Keep any needed immutable review library outside that target before cleanup; never replace or delete a library mapped by a running process. Preserve source, Git branches and concise logs/review evidence. Rebuild disposable executables and dependency objects when required.

```sh
# Run from the completed worktree after checking its processes have exited.
cargo clean --target-dir /absolute/path/to/completed/target
```

Check remaining space and target sizes between large validation runs. Do not retain previous full debug/release targets solely as evidence; source revisions, logs and explicitly preserved immutable libraries provide the reproducible record. Do not clean another active agent's target without coordinating ownership and stopped processes.
