# Contributing

We welcome contributions to Void eID!

## Development Guidelines

### Code Style

- **Rust**: Follow standard Rust idioms. Run `cargo fmt` and `cargo clippy` before pushing.
- **TypeScript**: Use Oxlint. Run `bun run lint` in the frontend directory.
- **Commits**: Use conventional commits (e.g., `feat: added wallet support`, `fix: token validation`).

### Frontend package management

Bun 1.3.14 is the canonical frontend package manager. From `src/frontend`, install the committed dependency graph with `bun install --frozen-lockfile` and run package scripts with `bun run <script>`. Use `bun add`, `bun remove`, or `bun update` for intentional dependency changes, and commit both `package.json` and `bun.lock`. Do not create or commit npm, Yarn, or pnpm lockfiles.

See [`src/frontend/README.md`](../src/frontend/README.md) for the complete frontend command reference.

### Workflow

1.  **Fork** the repository.
2.  **Create a branch** for your feature (`git checkout -b feature/amazing-feature`).
3.  **Implement** your changes.
4.  **Test** locally.
5.  **Submit a Pull Request**.

### Database Migrations

If you modify the database schema:

1.  Install `sqlx-cli`: `cargo install sqlx-cli`
2.  Add a migration: `sqlx migrate add <description>`
3.  Edit the generated `.sql` files.
4.  Run migrations: `sqlx migrate run`
5.  Update `sqlx-data.json` if using offline mode: `cargo sqlx prepare`

### Architecture Decisions

- Keep the backend stateless where possible (rely on JWT/DB).
- Frontend components should be reusable and adhere strictly to the "Design System".
- Avoid adding heavy dependencies unless necessary.

## Getting Help

If you get stuck, check the `/docs` folder or open an issue.
