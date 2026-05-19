FROM node:20-slim
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

RUN npm install
# Create seed mount point for the cache volume mechanism.
RUN mkdir -p node_modules_seed && chmod -R a+rwX node_modules node_modules_seed

CMD ["node", "--test", "tests/"]
