FROM ubuntu:latest

SHELL ["/bin/bash", "-c"]
ENV RUST_HOME /usr/local/lib/rust
ENV RUSTUP_HOME $RUSTUP_HOME/rustup
ENV CARGO_HOME $RUST_HOME/cargo
RUN mkdir $RUST_HOME
RUN chmod 0755 $RUST_HOME
ENV PATH $PATH:$CARGO_HOME/bin
RUN apt update
RUN apt install -y curl build-essential
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
RUN mkdir /app
WORKDIR /app
