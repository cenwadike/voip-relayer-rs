# # Builder stage
# FROM rust:1.82.0 AS builder

# # switch working directory to `app` (equivalent to `cd app`)
# # The `app` folder will be created by Docker in case it does not exist already.
# RUN mkdir -p /app
# WORKDIR /app

# # Install the required system dependencies for linking configuration 
# RUN apt update && apt install lld clang -y

# # Copy all files from working environment to Docker image
# COPY . .

# RUN cargo build --release

# # Runtime stage
# FROM debian:bullseye-slim AS runtime

# WORKDIR /app

# RUN apt-get update -y \
#     # Clean up
#     && apt-get autoremove -y \
#     && apt-get clean -y 

# # Copy the compiled binary from the builder environment
# # to our runtime environment
# COPY --from=builder /app/target/release/voip-relayer-rs voip-relayer-rs

# # When `docker run` is executed, launch the binary!
# ENTRYPOINT ["./voip-relayer-rs"]

# Use the official Rust image as the base image for building the application
# FROM rust:1.82.0 AS builder

# # Set the working directory
# WORKDIR /usr/src/app

# # Copy files to the working directory
# COPY . .

# # Build the application
# RUN cargo build --release

# # # Copy the compiled binary from the builder environment
# # # to our runtime environment
# COPY --from=builder /app/target/release/voip-relayer-rs voip-relayer-rs

# # Set the command to run the application
# ENTRYPOINT ["./voip-relayer-rs"]

# Use a Rust base image with Cargo installed
FROM rust:1.82.0 AS builder

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Create an empty src directory to trick Cargo into thinking it's a valid Rust project
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build the dependencies without the actual source code to cache dependencies separately
RUN cargo build --release

# Now copy the source code
COPY ./src ./src
COPY ./.env ./.env

# Build your application
RUN cargo build --release

# Start a new stage to create a smaller image without unnecessary build dependencies
FROM debian:bookworm-slim

# Set the working directory
WORKDIR /usr/src/app

# Copy the built binary from the previous stage
COPY --from=builder /usr/src/app//target/release/voip-relayer-rs voip-relayer-rs

# Command to run the application
CMD ["./voip-relayer-rs"]
