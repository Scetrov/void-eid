Use the `gh` CLI (GitHub CLI) to investigate the latest build failure on the 'main' branch (or [specify-branch-name]).

Please follow these steps:

1. **Fetch Workflows**: Use `gh run list --limit 5` to list recent workflow runs and identify the most recent failure.

2. **Get Logs**: Use `gh run view <run-id> --log-failed` to retrieve the failed job logs. Analyze the error messages, stack traces, and exit codes to pinpoint the root cause (e.g., test failure, linting error, or environment mismatch).

3. **Local Reproduction**:
   - Identify the command that failed in CI (e.g., `bun run test`, `cargo clippy`).
   - Attempt to run this command in a local terminal to see if the error is reproducible.

4. **CI vs. Local Analysis**:
   - If it fails locally, suggest a fix for the code.
   - If it passes locally but fails in CI, investigate environment differences (Rust/Bun versions, missing environment variables, or OS-specific paths).

5. **Propose Solution**: Provide a clear explanation of the fix and, if appropriate, prepare the changes to resolve the issue.

**Additional `gh` CLI commands that may be helpful**:
- `gh run list --workflow=ci.yml --limit 10` - List CI workflow runs
- `gh run view <run-id>` - View summary of a specific run
- `gh run view <run-id> --log` - View all logs for a run
- `gh run watch <run-id>` - Watch a run in progress
- `gh run rerun <run-id>` - Rerun a failed workflow

Acknowledge that CI environments can be finicky—if the logs suggest a transient infrastructure issue or a 'flaky' test, please highlight that specifically.
