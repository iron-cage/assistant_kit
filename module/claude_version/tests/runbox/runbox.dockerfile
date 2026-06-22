FROM rust:latest

ARG WORKSPACE_DIR=/workspace
WORKDIR $WORKSPACE_DIR

COPY . .

RUN cargo build && cargo test --no-run

RUN mkdir target_seed && chmod -R a+rwX target target_seed

CMD ["cargo", "test"]
