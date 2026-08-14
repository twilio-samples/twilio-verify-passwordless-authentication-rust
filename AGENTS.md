# Passwordless Authentication With Twilio Verify in Rust

This sample demonstrates how to implement passwordless authentication via OTP using Twilio Verify in a Rust/Axum web application.

## Environment Variables

Copy `.env.example` to `.env`. Never commit `.env`.

```bash
cp .env.example .env
```

| Variable | Where to find | Format |
| -------- | ------------- | ------ |
| `TWILIO_ACCOUNT_SID` | Console homepage or Admin dropdown (top right) → Account Management → Keys & Credentials → API Keys & Tokens | Starts with `AC` |
| `TWILIO_AUTH_TOKEN` | Console homepage or Admin dropdown (top right) → Account Management → Keys & Credentials → API Keys & Tokens → click to reveal | 32-char string. Treat as a password. |
| `TWILIO_VERIFY_SERVICE_SID` | Console → Verify → Services | Starts with `VA` |

## Commands

```bash
# Install
cargo build

# Run
cargo run
```

## Project Structure

- `src/main.rs` — route handlers for the sign-in form, OTP dispatch, and OTP verification
- `templates/forms/` — Handlebars templates for sign-in, OTP entry, and verification status
- `Cargo.toml` — project dependencies
- `.env.example` — environment variable template
- `assets/` — static assets served at `/assets`

## Agent Boundaries

**Always:**
- Confirm `.env` is configured before running any command
- Use the Environment Variables section to guide the user to each credential — don't ask them to find values without direction
- Confirm the app is running before asking the user to test it

**Never:**
- Run the app with missing or placeholder credentials
- Hardcode credentials or phone numbers in source files
- Skip the `cp .env.example .env` step

## Verify It's Working

1. Open http://localhost:8080 in your browser, enter your phone number in E.164 format (e.g., `+15551234567`), and submit the form.
2. You'll receive an SMS with a 6-digit OTP code — enter it on the verification page and submit. A success message confirms the app is working end-to-end.

## Twilio Resources

- [Twilio Console](https://console.twilio.com) — credentials, phone numbers, webhook configuration
- [Twilio Verify documentation](https://www.twilio.com/docs/verify/api)
- [Twilio Rust helper library (rustlio)](https://crates.io/crates/rustlio)
