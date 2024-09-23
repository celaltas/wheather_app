CARGO := cargo
SQLX := sqlx

.PHONY: all
all: help

# Build for production
.PHONY: build-prod
build-prod: db-create migrate
	@echo "Building for production..."
	$(CARGO) build --release

# Build for development
.PHONY: build-dev
build-dev: db-create migrate
	@echo "Building for development..."
	$(CARGO) build

# Clean the project
.PHONY: clean
clean:
	$(CARGO) clean

# Create the database
.PHONY: db-create
db-create:
	$(SQLX) db create || echo "Database already exists"

# Run migrations
.PHONY: migrate
migrate:
	$(SQLX) migrate run

# Help target
.PHONY: help
help:
	@echo "Available targets:"
	@echo "  build-prod  - Build the project in production mode"
	@echo "  build-dev   - Build the project in development mode"
	@echo "  clean       - Clean the project"
	@echo "  db-create   - Create the database (automatically run before build)"
	@echo "  migrate     - Run database migrations (automatically run before build)"
	@echo "  help        - Display this help message"
