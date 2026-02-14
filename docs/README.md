# VoID Electronic Identity (eID)

![VoID eID Logo](../src/frontend/public/logo.png)


A modern web application integrating Discord authentication with Sui blockchain wallet verification, featuring a Rust backend and a React frontend.

## Overview

Void eID provides a seamless way to link Discord identities with Sui Wallets. It uses a robust Rust backend for secure authentication and state management, and a modern React frontend for user interaction.

## Project Structure

- **Backend** (`src/backend`):
  - Written in Rust using [Axum](https://github.com/tokio-rs/axum).
  - Uses SQLite for data persistence via [SQLx](https://github.com/launchbadge/sqlx).
  - Implements OpenAPI documentation with [Utoipa](https://github.com/juhaku/utoipa) and [Scalar](https://github.com/scalar/scalar).
  - Handles Discord OAuth2 flow and Sui Wallet signature verification.

- **Frontend** (`src/frontend`):
  - Built with [Vite](https://vitejs.dev/) and [React 19](https://react.dev/).
  - Integrates [Sui dApp Kit](https://sdk.mystenlabs.com/dapp-kit) for wallet connections.
  - Uses [TanStack Router](https://tanstack.com/router) for type-safe routing.
  - Styled with a custom design system (see [Design System](./design-system.md)).

## Quick Start

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Node.js](https://nodejs.org/) & [Bun](https://bun.sh/)
- [Just](https://github.com/casey/just) (optional, strictly recommended for task running if added later)
- Discord Application Credentials (see [Discord Setup](./discord-setup.md))

### Backend Setup

1. Navigate to `src/backend`.
2. Copy `.env.example` to `.env` and populate **required** variables:
   ```env
   DATABASE_URL=sqlite:void-eid.db
   DISCORD_CLIENT_ID=your_discord_client_id
   DISCORD_CLIENT_SECRET=your_discord_client_secret
   DISCORD_REDIRECT_URI=http://localhost:5038/api/auth/discord/callback
   JWT_SECRET=your_jwt_secret_minimum_32_characters
   IDENTITY_HASH_PEPPER=your_random_pepper_string
   INTERNAL_SECRET=your_random_internal_secret
   PORT=5038
   ```
   **Generate secrets** using: `openssl rand -base64 32`

   If running **Mumble**, also set:
   ```env
   ICE_SECRET_READ=your_ice_read_secret
   ICE_SECRET_WRITE=your_ice_write_secret
   ICE_SECRET=your_ice_read_secret
   ```
3. Run the backend:
   ```bash
   cargo run
   ```
   The API will be available at `http://localhost:5038`.
   API Documentation is available at `http://localhost:5038/docs`.

### Frontend Setup

1. Navigate to `src/frontend`.
2. Install dependencies:
   ```bash
   bun install
   ```
3. Run the development server:
   ```bash
   bun run dev
   ```
   The app will be available at `http://localhost:5173`.

## Documentation Index

- [Backend Documentation](./backend.md) - Detailed API and architecture info.
- [Frontend Documentation](./frontend.md) - Component structure and state management.
- [Deployment Guide](./deployment.md) - How to build and deploy.
- [Release Process](./release-process.md) - Automated release workflow.
- [Design System](./design-system.md) - UI styles and tokens.
- [Contributing](./contributing.md) - Development guidelines.
