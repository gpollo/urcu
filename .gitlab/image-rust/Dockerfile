FROM ubuntu:oracular-20240918
ARG RUST_VERSION=nightly-2024-10-08

ENV DEBIAN_FRONTEND=noninteractive TZ=America/Montreal
RUN apt-get update -y \
 && apt-get install -y pkg-config curl git clang libssl-dev pkg-config \
 && curl -sSL "https://raw.githubusercontent.com/upciti/wakemeops/main/assets/install_repository" | bash \
 && apt-get install -y glab \
 && curl https://sh.rustup.rs -sSf | bash -s -- -y \
 && . "$HOME/.cargo/env" \
 && until rustup install --profile=minimal "${RUST_VERSION}"; do echo; done \
 && until rustup default "${RUST_VERSION}"; do echo; done \
 && until rustup component add clippy; do echo; done \
 && until rustup component add rustfmt; do echo; done \
 && until rustup component add llvm-tools-preview; do echo; done \
 && rustup toolchain uninstall stable \
 && cargo install --git https://github.com/rustsec/rustsec.git --rev fc8d206cc2 --locked cargo-audit \
 && cargo install cargo-nextest@0.9.81 --locked \
 && cargo install cargo-llvm-cov@0.6.15 --locked \
 && cargo install tomlq@0.1.6 --locked
ENV PATH="/root/.cargo/bin:${PATH}"