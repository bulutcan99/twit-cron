# Use the official Rust image from Docker Hub
FROM rust:latest

# Set the working directory inside the container
WORKDIR /usr/src/myapp

# Copy the project files into the container
COPY . .

# Build the project
RUN cargo build --release

# Set the default command to run the application
CMD ["./target/release/twitter-cron"]
