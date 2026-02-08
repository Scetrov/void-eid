## Plan: Migrate backend Docker builds to debian-slim with pre-built glibc binary

**TL;DR**: Eliminate the in-Docker Rust compilation entirely. The CI backend job already builds a glibc binary on `ubuntu-latest` — reuse that binary in a simple `debian:bookworm-slim` runtime image instead of compiling from scratch inside `rust:alpine`. This fixes a latent glibc/musl mismatch in the release workflow, removes ~50 lines of Dockerfile complexity, and cuts Docker build time from minutes to seconds. The `push-images` job becomes an artifact consumer rather than a full rebuild.

**Steps**

1. **Always build the release binary in the backend CI job** (`.github/workflows/ci.yml`)
   - Remove the `if: ${{ inputs.is_release }}` guard from both the "Build release binary" step and the "Upload backend binary" step
   - The release binary is now always built and uploaded as the `backend-bin` artifact, alongside `stub_api`
   - This costs ~1 extra minute in the backend job but eliminates an entire rebuild in Docker

2. **Replace the main Dockerfile with a simple debian-slim copy** (`src/backend/Dockerfile`)
   - Replace the entire 51-line multi-stage `rust:alpine` build with a ~12-line `debian:bookworm-slim` image that just copies a pre-built binary
   - Install runtime deps: `ca-certificates`, `libssl3`, `libsqlite3-0`
   - Remove `# syntax=docker/dockerfile:1` directive (no BuildKit cache mounts needed)
   - The binary is passed in via the Docker build context (not built inside Docker)

3. **Update Dockerfile.release to use debian-slim** (`src/backend/Dockerfile.release`)
   - Switch from `alpine:latest` to `debian:bookworm-slim`
   - Replace `apk add` with `apt-get install -y ca-certificates libssl3 libsqlite3-0`
   - Remove `libgcc` (included in debian-slim by default)
   - This **fixes the existing glibc/musl mismatch bug** where release builds fail because a glibc binary is placed in an Alpine container

4. **Unify Dockerfile and Dockerfile.release** — since both now do the same thing (COPY a pre-built binary into debian-slim), consolidate into a single `Dockerfile` and delete `Dockerfile.release`. Update the release workflow to reference `Dockerfile` instead of `Dockerfile.release`.

5. **Restructure the CI `push-images` job** (`.github/workflows/ci.yml`)
   - Add `needs: [backend, frontend, murmur]` so it runs after build jobs complete
   - Remove the matrix entry for `backend` from the service matrix (it now uses a different flow)
   - Add a dedicated backend push step that:
     - Downloads the `backend-bin` artifact into a temp directory
     - Runs `docker/build-push-action` with that directory as context and the new `Dockerfile`
   - Keep `frontend` and `murmur` in the matrix — they still build from source in Docker (frontend is just nginx static files, murmur needs Alpine's `mumble-server` package)
   - Remove `SCCACHE_GHA_ENABLED`, `RUSTC_WRAPPER` env vars (no longer needed — no Rust compilation in this job)

6. **Update the release workflow** (`.github/workflows/release.yml`)
   - Change the backend `build-and-push` step to reference `src/backend/Dockerfile` instead of `src/backend/Dockerfile.release` (since they're now unified)
   - Verify the artifact download path places `void-eid-backend` where the Dockerfile expects it

7. **Update docker-compose.yml** (`docker-compose.yml`)
   - The dev compose file uses `Dockerfile.dev` for the backend — this is unaffected (already uses glibc `rust:latest`)
   - Verify no references to `Dockerfile.release` exist in compose files

8. **Update deployment docs** (`docs/deployment.md`)
   - Remove the mention of "build static musl binary" option since the project now standardizes on glibc/debian-slim
   - Update any Dockerfile examples to reflect the new simplified structure
   - Note that production images use `debian:bookworm-slim` as the runtime base

9. **Remove the Docker image build from the backend CI job** (`.github/workflows/ci.yml`)
   - The "Build Docker image" step in the backend job currently exists as a validation step (`push: false`)
   - This step is no longer meaningful since the Dockerfile is now a trivial COPY — there's nothing to validate
   - Remove it (or optionally keep it as a smoke test, but it adds ~5s of overhead for little value)

10. **Clean up CI backend job permissions** (`.github/workflows/ci.yml`)
    - The `actions: write` permission was needed for Docker GHA cache and sccache — check if it's still needed for `Swatinem/rust-cache` and `sccache-action` (both still run for the host cargo steps). If so, keep it; if not, reduce to `actions: read`

**Verification**

- Push the changes and confirm CI passes: backend job builds + uploads binary, `push-images` downloads and packages it, E2E tests still work with `stub_api`
- Manually run `docker build -t test-backend src/backend/` locally after placing a built binary in `src/backend/` to verify the image starts correctly
- Trigger a test release (or dry-run the release workflow) to verify `Dockerfile` works in place of `Dockerfile.release`
- Check image size: `docker images test-backend` — expect ~90-110MB total (80MB debian-slim + ~15-25MB binary)
- Run the container: `docker run --rm test-backend ./void-eid-backend --help` to verify dynamic linking is satisfied (no missing `.so` errors)

**Decisions**

- **Pre-built binary over in-Docker build**: chosen for CI speed, consistency (tested binary = shipped binary), and to fix the existing glibc/musl mismatch in releases
- **Keep native-tls**: requires `libssl3` in the runtime image but avoids a Cargo.toml change with broader implications
- **Keep nginx:alpine for frontend**: no binary compatibility concern for static files, smaller image
- **Keep murmur on Alpine**: depends on Alpine's `mumble-server` package
- **Unify Dockerfile and Dockerfile.release**: they become identical after migration, no reason to maintain two files
