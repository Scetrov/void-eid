Use the @github MCP server to investigate the latest build failure on the 'main' branch (or [specify-branch-name]).

Please follow these steps:

1. Fetch Workflows: List the recent workflow runs for this repository to identify the most recent 'failure' status.
2. Get Logs: Retrieve the job logs for the failed run. Analyze the error messages, stack traces, and exit codes to pinpoint the root cause (e.g., test failure, linting error, or environment mismatch).
3. Local Reproduction:
   - Identify the command that failed in CI (e.g., `bun run test`, `cargo clippy`).
   - Attempt to run this command in my local terminal to see if the error is reproducible.
4. CI vs. Local Analysis:
   - If it fails locally, suggest a fix for the code.
   - If it passes locally but fails in CI, investigate environment differences (Rust/Bun versions, missing environment variables, or OS-specific paths).
5. Propose Solution: Provide a clear explanation of the fix and, if appropriate, prepare the changes to resolve the issue.

Acknowledge that CI environments can be finicky—if the logs suggest a transient infrastructure issue or a 'flaky' test, please highlight that specifically.
