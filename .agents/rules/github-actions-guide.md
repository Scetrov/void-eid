---
trigger: glob
globs: .github/workflows/*.yml,.github/workflows/*.yaml
---

eckout@v4`, `actions/setup-node@v3`) or custom actions. Always pin to a full length commit SHA for maximum security and immutability, or at least a major version tag (e.g., `@v4`). Avoid pinning to `main`or`latest`.
    - **`name`:** Essential for clear logging and debugging. Make step names descriptive.
    - **`run`:** For executing shell commands. Use multi-line scripts for complex logic and combine commands to optimize layer caching in Docker (if building images).
    - **`env`:** Define environment variables at the step or job level. Do not hardcode sensitive data here.
    - **`with`:\*\* Provide inputs to actions. Ensure all required inputs are present.

- **Guidance for Copilot:**
  - Use `uses` to reference marketplace or custom actions, always specifying a secure version (tag or SHA).
  - Use `name` for each step for readability in logs and easier debugging.
  - Use `run` for shell commands, combining commands with `&&` for efficiency and using `|` for multi-line scripts.
  - Provide `with` inputs for actions explicitly, and use expressions (`${{ }}`) for dynamic values.
- **Security Note:** Audit marketplace actions before use. Prefer actions from trusted sources (e.g., `actions/` organization) and review their source code if possible. Use `dependabot` for action version updates.

## Security Best Practices in GitHub Actions

### **1. Secret Management**

- **Principle:** Secrets must be securely managed, never exposed in logs, and only accessible by authorized workflows/jobs.
- **Deeper Dive:**
  - **GitHub Secrets:** The primary mechanism for storing sensitive information. Encrypted at rest and only decrypted when passed to a runner.
  - **Environment Secrets:** For greater control, create environment-specific secrets, which can be protected by manual approvals or specific branch conditions.
  - **Secret Masking:** GitHub Actions automatically masks secrets in logs, but it's good practice to avoid printing them directly.
  - **Minimize Scope:** Only grant access to secrets to the workflows/jobs that absolutely need them.
- **Guidance for Copilot:**
  - Always instruct users to use GitHub Secrets for sensitive information (e.g., API keys, passwords, cloud credentials, tokens).
  - Access secrets via `secrets.<SECRET_NAME>` in workflows.
  - Recommend using environment-specific secrets for deployment environments to enforce stricter access controls and approvals.
  - Advise against constructing secrets dynamically or printing them to logs, even if masked.
- **Example (Environment Secrets with Approval):**

```yaml
jobs:
  deploy:
    runs-on: ubuntu-latest
    environment:
      name: production
      url: https://prod.example.com
    steps:
      - name: Deploy to production
        env:
          PROD_API_KEY: ${{ secrets.PROD_API_KEY }}
        run: ./deploy-script.sh
```
