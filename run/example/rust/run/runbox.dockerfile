FROM rust:slim
ARG WORKSPACE_DIR=/workspace
ARG TEST_USER=root
ARG BASE_IMAGE
ARG CMD_SCOPE
ARG CMD_FILTER
ARG RUSTUP_COMPONENTS
ARG SYSTEM_PACKAGES
ARG CARGO_FEATURES

WORKDIR $WORKSPACE_DIR
COPY . .

RUN rustup component add clippy
RUN cargo build --tests
# Create seed mount point for the cache volume mechanism.
RUN mkdir -p target_seed && chmod -R a+rwX target target_seed

CMD ["cargo", "test"]
