+++
title = "Privacy Policy"
author = "Richard Slater"
+++

**Effective Date:** 2026-02-13
**Last Updated:** {{ .lastUpdated }}

## 1. Introduction

Welcome to VoID Electronic Identity (eID) ("we," "our," or "us"). We respect your privacy and are committed to protecting your personal data. This Privacy Policy explains how we collect, use, and safeguard your information when you use our web application to link your Discord identity with your Sui Wallet.

## 2. Data Controller & Hosting

* **Hosting Provider:** The Service is hosted on private servers provided by **OVH Cloud** located in **France**. Consequently, your primary data resides within the European Union (EU) and is protected under the General Data Protection Regulation (GDPR).
* **Content Delivery & Security:** We use **Cloudflare** as a Content Delivery Network (CDN) and for secure tunnel connectivity. While primary storage is in France, transit data may pass through Cloudflare’s global network to ensure low latency and security.

## 3. Information We Collect

We practice data minimization and only collect what is strictly necessary for the Service to function:

### A. Information You Provide

* **Discord Identity:** When you authenticate via Discord, we store your unique Discord ID and public profile information (username, avatar).
* **Wallet Information:** We store your Sui Wallet public address and the cryptographic signature provided during the verification process to prove ownership.

### B. Automatically Collected Information

* **Technical Log Data:** Our servers and Cloudflare may automatically log your IP address, browser type, and access times for security purposes (e.g., DDoS protection, rate limiting) in line with the [Cloudflare Privacy Policy](https://www.cloudflare.com/en-gb/privacypolicy/).
* **Blockchain Data:** Your wallet interactions are public transactions on the Sui Blockchain. We do not control the transparency of the blockchain itself, please review the [Sui Privacy Policy](https://sui.io/privacy-policy) for more information.

## 4. How We Use Your Data

We use your data for the following specific purposes:

1. **Authentication:** To verify your identity using Discord OAuth2.
2. **Verification:** To cryptographically prove that a specific Discord user owns a specific Sui Wallet address.
3. **Access Control:** To manage permissions and roles (e.g., "Tribes") within our ecosystem based on your linked identity.
4. **Security:** To detect and prevent fraud, spam, or unauthorized access.

## 5. Data Sharing and Processors

We do not sell your personal data. We share data only with the following infrastructure providers necessary to operate the Service:

* **OVH Cloud (France):** Physical hosting and database storage.
* **Cloudflare (Global):** CDN, DDoS protection, and secure tunneling.
* **Discord:** We interact with Discord solely for authentication; by using the service you are allowing the admins of the tribe view your Discord ID for the purposes of granting permissions.
* **Sui Network:** The association of your wallet address is a public blockchain concept; however, our internal database linking that address to your Discord ID is private and only accessible to admins of the tribe(s) you are a member of.

## 6. Your Rights (GDPR & Global)

Under the GDPR and similar frameworks, you have the right to:

* **Access:** Request a copy of the data we hold about you (Discord ID and linked Wallet), you can review this information in the service homepage (`/home`).
* **Rectification:** Update your data (typically done by re-syncing your Discord profile).
* **Erasure ("Right to be Forgotten"):** You may unlink your wallet and request deletion of your account. We support "soft-deletes" and unlinking capabilities.
* **Portability:** Request your data in a structured, machine-readable format.

To exercise these rights, please contact us using the compliance request form.
