# Security and Vulnerability Reporting Guide

This document outlines the policies for reporting security vulnerabilities and the versioning support provided by the Void eID team. We take the security of our Rust (Axum/SQLx), React, and Docker-based infrastructure seriously.

## 1. Supported Versions

To ensure the highest level of security, **only the most recent minor version** of Void eID is supported for security updates and patches. Users are strongly encouraged to stay on the latest release to ensure they are protected against known vulnerabilities.

| Version | Security Support |
| --- | --- |
| **Current (e.g., 0.2.x)** | :white_check_mark: **Supported** |
| **Legacy (e.g., 0.1.x)** | :x: **Unsupported** |

Release artifacts, including Binaries and Docker Images, are automatically patched with the correct version number during the automated release process.

## 2. Reporting a Vulnerability

If you discover a security vulnerability, please do not disclose it publicly until it has been addressed. Public disclosure puts all users at risk.

### Preferred Reporting Method

Please use **GitHub's vulnerability reporting** feature to submit your findings. This allows for a private, encrypted communication channel between the reporter and the maintainers.

### Reporting Scope

We are particularly interested in reports concerning:

* **Backend (Rust):** Authentication logic, JWT handling errors, SQL injection in SQLx queries, and internal API secret exposure.
* **Frontend (React):** Insecure environment variable usage, routing guard bypasses, and wallet connection vulnerabilities.
* **Infrastructure:** Insecure defaults in `docker-compose.yml`, GitHub Actions CI/CD workflow leaks, and Murmur/Mumble authenticator script flaws.

## 3. Evaluation Criteria

Vulnerabilities are assessed based on the **OWASP Top 10:2025** standards and mapped to the **MITRE ATT&CK Framework**. High-priority checks include:

* **Broken Access Control (A01:2025):** Validation failures in session extractors or admin-only routes.
* **Cryptographic Failures (A02:2025):** Hardcoded secrets, weak JWT signing, or weak Sui wallet signature verification.
* **Injection (A03:2025):** Non-parameterized SQLx queries.
* **Identification Failures (A07:2025):** CSRF vulnerabilities in Discord OAuth2 flows.

## 4. Response and Disclosure Process

1. **Acknowledgment:** The team will acknowledge receipt of the report through GitHub's reporting tool.
2. **Triage:** Maintainers will investigate the issue and determine the severity (Critical, High, Medium, or Low).
3. **Remediation:** A fix will be developed for the **current supported version**. Because we only support the most recent version, fixes will not be backported to older minor releases.
4. **Release:** The fix will be deployed via the automated GitHub Actions release workflow.
5. **Public Disclosure:** Once a fix is available and users have had a reasonable window to update, a Security Advisory will be published.

## 5. Security Audit Format

When submitting a report, please try to follow the standardized security audit format used by our internal researchers:

* **ID:** A unique identifier (e.g., SEC-XX).
* **Location:** Specific file path and line number (e.g., `src/backend/src/auth.rs:45`).
* **Description:** A detailed explanation of the vulnerability and its potential impact.
* **MITRE ATT&CK Mapping:** Relevant technique ID.
* **Remediation Guidance:** Actionable steps or code snippets to resolve the issue.
