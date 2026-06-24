# Rust Email OTP Service

A production-ready, high-performance Email One-Time Password (OTP) microservice written in Rust using the Axum framework. 

This service handles generating, securely storing (via Redis), emailing, and verifying 6-digit OTP codes. It is built from the ground up to prevent brute-force attacks and abuse using advanced rate-limiting logic.

## 🚀 Key Features

* **Blazing Fast**: Built on top of `tokio` and `axum`.
* **Advanced Rate Limiting**: 
  * Generates use **Exponential Backoff** to strictly prevent email spam (e.g., 30s cooldown, then 60s, then 120s, up to 8.5 hours).
  * Verification enforces a maximum of 10 failed guesses per minute to prevent brute-force attacks.
* **Smart Security**: 
  * Strict payload validation for email formatting using the `validator` crate.
  * Permissive CORS enabled for frontend integration.
  * Security headers injected via `tower-http` (`Strict-Transport-Security`, `X-Content-Type-Options`, `X-Frame-Options`).
  * Runs as a secure `non-root` user within the production Docker container.
* **Auto-Expiring OTPs**: Powered natively by Redis TTL.
* **HTML Email Templates**: Emails are beautifully rendered using the `askama` compiled templating engine.
* **Graceful Shutdown**: Traps `SIGINT` and `SIGTERM` to safely drain active connections before shutting down.
* **Interactive OpenAPI Docs**: Self-documenting API via `utoipa` and Swagger UI.

---

## 🛠️ Tech Stack

* **Language**: Rust
* **Web Framework**: [Axum](https://github.com/tokio-rs/axum)
* **Storage**: Redis (via `deadpool-redis`)
* **Email Client**: `lettre`
* **Templates**: `askama`
* **Validation**: `validator`
* **Docs**: `utoipa` (Swagger UI)

---

## 📦 Prerequisites

* [Rust & Cargo](https://rustup.rs/) (v1.75+)
* [Docker & Docker Compose](https://www.docker.com/) (For local Redis and Mailpit)

---

## 💻 Local Development Setup

### 1. Start the Infrastructure
We use `docker-compose` to spin up a local Redis instance and **Mailpit** (a local SMTP server for catching emails without actually sending them).
```bash
docker-compose up -d
```
*Note: Mailpit's web interface will be available at [http://localhost:8025](http://localhost:8025).*

### 2. Configure Environment
Copy the example environment file:
```bash
cp .env.example .env
```

### 3. Run the Server
```bash
cargo run
```
The server will start on `http://localhost:3000`.

---

## 📖 API Documentation

Once the server is running, you can explore and test the API using the interactive Swagger UI:

👉 **[http://localhost:3000/swagger-ui](http://localhost:3000/swagger-ui)**

### Endpoints

#### `POST /api/v1/otp/generate`
Generates an OTP and sends it via email.
* **Payload**: `{"email": "user@example.com"}`
* **Success (200)**: Returns `retry_after_seconds` letting you know when the user can request another code.
* **Rate Limited (429)**: Returns the remaining cooldown penalty.

#### `POST /api/v1/otp/verify`
Verifies a previously generated OTP.
* **Payload**: `{"email": "user@example.com", "code": "123456"}`
* **Success (200)**: OTP successfully verified and deleted from memory.
* **Unauthorized (401)**: OTP was invalid or expired.
* **Rate Limited (429)**: Too many failed guesses.

---

## 🐳 Production Deployment

A highly-optimized, multi-stage `Dockerfile` is included for production deployments.

### Build the Image
```bash
docker build -t mfa-service .
```

### Run the Image
```bash
docker run -p 3000:3000 --env-file .env mfa-service
```
*Note: Be sure to update your `.env` to point to a real production Redis instance and SMTP provider (like Resend, SendGrid, or AWS SES) before deploying.*
