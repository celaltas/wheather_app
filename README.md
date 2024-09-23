# Weather App

## Overview
Brief description of the project and its purpose.

## Key Features

- **API Documentation**:  
  The `/docs` endpoint provides detailed information about the API endpoints and how to use them. This makes it easy for developers to interact with the API and reduces errors.

- **Configuration**:  
  All application settings are managed in a central configuration folder, allowing for different settings to be used in various environments (development, testing, production).

- **JWT Authentication**:  
  The application uses JSON Web Tokens (JWT) for security, preventing unauthorized access.

- **Weather Data**:  
  The application retrieves weather data from an external service, ensuring that the information is up-to-date and accurate.

- **Caching**:  
  A caching mechanism is employed for frequent requests from the same IP address, reducing server load and improving response times.

- **CORS, Rate Limiting, Tracing, TLS**:  
  The application uses CORS (Cross-Origin Resource Sharing), rate limiting, tracing, and TLS (Transport Layer Security) to enhance security, performance, and observability.

- **Graceful Shutdown**:  
  The application can gracefully shut down, ensuring that ongoing requests are completed before closure, thus maintaining uninterrupted service.

- **Integration Tests**:  
  Integration tests are written to verify that all components of the application work together correctly. These tests run on an independent application and database.

## Getting Started

### Prerequisites
List any prerequisites or dependencies needed to run the application.

### Installation
There is a Makefile to build for production or development environments. Run the application with:

```bash
cargo run --release  # For production
cargo run            # For development


