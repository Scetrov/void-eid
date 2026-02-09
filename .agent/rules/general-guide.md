---
trigger: always_on
globs: *
---

## Constitution

1. This project requires that all commits are signed, **NEVER** attempt to disable GPG.
2. Ensure that changes are clean and canonical, **ALWAYS** add files to git and then run `pre-commit run --all` to perform linting, build and test.
3. Unit tests are mandatory, **ALWAYS** include comprehensive unit tests that test both the happy-path and the not-happy-path.
4. UI tests **ALWAYS** go with changes to the User Experience.
