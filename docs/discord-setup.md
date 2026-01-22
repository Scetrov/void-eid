# Discord Application Setup Guide

To enable Discord Login for Void eID, you must create a Discord Application in the Developer Portal.

## 1. Create a New Application

1.  Go to the [Discord Developer Portal](https://discord.com/developers/applications).
2.  Click **New Application**.
3.  Enter a name (e.g., "Void eID Dev") and accept the terms.
4.  Click **Create**.

## 2. Configure OAuth2

1.  In the sidebar, navigate to **OAuth2**.
2.  Under **Redirects**, click **Add Redirect**.
3.  Add the callback URL for your local environment:
    ```
    http://localhost:5038/api/auth/discord/callback
    ```
    _(Note: If your backend runs on a different port, update this accordingly.)_
4.  Click **Save Changes**.

## 3. Get Credentials

1.  Stay on the **OAuth2** page (or go to **General Information** for the App ID).
2.  Copy the **Client ID**.
3.  Click **Reset Secret** to generate a new **Client Secret**. Copy it immediately.

## 4. Configure Application

Update your `src/rust/.env` file with these values:

```env
DISCORD_CLIENT_ID=YOUR_CLIENT_ID
DISCORD_CLIENT_SECRET=YOUR_CLIENT_SECRET
# This is usually handled automatically, but if you need to override:
# REDIRECT_URI=http://localhost:5038/api/auth/discord/callback
```

## Required Scopes

The application currently uses the following scope:

- `identify` (allows access to username, avatar, and ID).

No bot permissions are required for this login flow.
