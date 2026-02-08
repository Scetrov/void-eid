---
description: Security Audit Prompt: VoID Electronic Identity (eID)
---

**Role:** You are a Senior Security Researcher and DevSecOps Engineer specializing in Rust (Axum/SQLx), React, and Docker security.

**Objective:** Conduct a deep-dive security audit of the VoID eID project. Evaluate the codebase against the **OWASP Top 10:2025**, and supplemental secure coding guidelines from **Microsoft, Google, CIS, NIST, NCSC, and CISA**.

**Scope of Analysis:**

1. **Backend (Rust):** Authentication logic, JWT handling, SQLx queries, CORS configuration, and internal API secrets.
2. **Frontend (React/Vite):** Environment variable usage, routing guards, and wallet connection security.
3. **Infrastructure:** `docker-compose.yml`, GitHub Actions CI/CD workflows, and Murmur/Mumble authenticator scripts.

**Instructions:**

1. Identify specific violations with **file paths** and **line numbers**.
2. Provide a description of the vulnerability and its potential impact.
3. Map each finding to the **MITRE ATT&CK Framework**.
4. Provide actionable **remediation guidance** for developers.
5. Summarize findings in the **Standardized Security Audit Report** format provided below.

---

### Audit Criteria & High-Priority Checks

- **OWASP A01:2025 – Broken Access Control:** Check `src/backend/src/auth.rs` and `roster.rs`. Ensure `AuthenticatedUser` extractor properly validates sessions and that admin-only routes (like `grant_admin`) verify the `is_admin` flag in the database, not just the token.
- **OWASP A02:2025 – Cryptographic Failures:** Evaluate the JWT implementation in `auth.rs`. Check for hardcoded secrets, weak signing algorithms, or lack of expiration validation. Inspect Sui wallet signature verification in `wallet.rs`.
- **OWASP A03:2025 – Injection:** Verify that all SQLx queries use parameterized inputs and not string interpolation (check `01_init.sql` through `04_unique_wallets_address.sql` and `db.rs`).
- **OWASP A05:2025 – Security Misconfiguration:** Review `docker-compose.yml` for insecure defaults (e.g., SQLite file permissions, exposed ports). Check `main.rs` for overly permissive CORS policies.
- **OWASP A07:2025 – Identification and Authentication Failures:** Analyze the Discord OAuth2 flow in `auth.rs`. Check for "State" parameter usage to prevent CSRF in OAuth.
- **Supply Chain Security:** Review `.github/workflows/ci.yml` for insecure action versions or lack of integrity checks.

---

### Standardized Security Audit Report Format

#### 1. Executive Summary

- Overall Risk Rating (Critical/High/Medium/Low)
- Summary of top 3 critical risks.

#### 2. Detailed Findings

| ID     | Vulnerability Name    | Severity | Location (File:Line)       | OWASP 2025 Mapping | MITRE ATT&CK ID |
| ------ | --------------------- | -------- | -------------------------- | ------------------ | --------------- |
| SEC-01 | [e.g., SQL Injection] | Critical | `src/backend/src/db.rs:45` | A03:2025           | T1190           |

**Description:** [Detailed explanation of the vulnerability]
**Impact:** [What an attacker can achieve]
**Guidance/Fix:** [Code snippet or configuration change to resolve the issue]

#### 3. Infrastructure & CI/CD Review

- **Docker Security:** Analysis of `Dockerfile` and `docker-compose.yml`.
- **CI/CD Pipeline:** Analysis of `ci.yml` (e.g., secrets handling, binary signing).

#### 4. Compliance Check

- Adherence to **NIST SP 800-53** (Access Control) or **CIS Benchmarks** (Docker/Linux).
