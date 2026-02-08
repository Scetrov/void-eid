---
description: Addresses GitHub Pull Request Comments
---

## Purpose

This prompt guides an AI agent to resolve Pull Request feedback. The agent must follow a strict linear flow for every unresolved comment: **Identify the issue, implement the fix, and reply directly to the original comment thread with a resolution summary.**

## Prerequisites

* GitHub CLI (`gh`) installed and authenticated.
* Current branch matches the PR branch.
* `GH_PAGER` set to `cat`.

## Execution Flow

### Phase 1: Context & Discovery

1. **Determine Target PR**: Extract number from `$ARGUMENTS` or detect via `gh pr view --json number`.
2. **Verify Branch**: Ensure `git branch --show-current` matches the PR `headRefName`. **Stop if they differ.**
3. **Fetch Unresolved Threads**: Execute the GraphQL query to find nodes where `isResolved: false`.

### Phase 2: The "Fix & Reply" Loop

For **each** unresolved comment thread identified in Phase 1, the agent MUST perform these steps in order:

#### 1. Analysis

* Read the file context using `read_file` (±20 lines around the `line` indicated in the thread).
* Categorize the request (Code Change, Docs, Test, or Clarification).

#### 2. Implementation

* Apply the fix using `replace_string_in_file` or `multi_replace_string_in_file`.
* **Validation**: Run `cargo fmt`, `cargo clippy`, and `cargo test` (or relevant language equivalents) to ensure the fix is valid.

#### 3. Commit

* Create an atomic commit for the fix.
* **Commit Message**: Reference the PR and the specific feedback (e.g., `fix: address review comment on <file> regarding <topic>`).

#### 4. The Response (Mandatory)

* You **must** reply to the specific comment thread so the reviewer is notified.
* **Command**:
```bash
# Note: Use the 'id' from the reviewThread or the 'databaseId' of the last comment in the thread
gh api --method POST repos/{owner}/{repo}/pulls/{pr_number}/comments/{comment_id}/replies \
  -f body="✅ **Addressed in [SHA]**

  Brief description of the fix:
  - [Specific change 1]
  - [Specific change 2]"

```



### Phase 3: Finalization

1. **Push**: `git push origin HEAD`.
2. **Review Request**: If the `reviewDecision` was `CHANGES_REQUESTED`, prompt the user to re-request review: `gh pr edit <number> --add-reviewer <username>`.
3. **Summary Report**: Provide a table of addressed comments, their associated commits, and confirmation that replies were posted.

## Structured TODO List Template

The agent should maintain this internal state to track progress:

| Thread ID | Location | Status | Action Taken | Reply Posted? |
| --- | --- | --- | --- | --- |
| `ref_123` | `src/main.rs:42` | ✅ Fixed | Added bounds check | Yes |
| `ref_456` | `README.md:10` | ⏳ Pending | - | No |

## Error Handling

* **Missing Permissions**: If `gh api` fails with 403, notify the user immediately.
* **Ambiguous Comments**: If a comment is unclear, do not guess. Reply to the thread asking for clarification and mark as "Pending" in the summary.
* **Code Conflict**: If the file has changed significantly since the comment (outdated), notify the user before applying fixes.
**Would you like me to generate the specific GraphQL query optimized for fetching the `databaseId` needed for those replies?**
