FROM rust

ARG CURRENT_NODE="node0"
ENV CURRENT_NODE ${CURRENT_NODE}

RUN apt-get update && apt-get -y install libsodium18 libsodium-dev pkg-config
WORKDIR /src/app
COPY . /src/app
RUN cargo install --path dmbc-node

CMD ["/usr/local/cargo/bin/dmbc-node"]
