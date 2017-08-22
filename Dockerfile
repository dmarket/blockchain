FROM rust
RUN apt-get update && apt-get -y install libsodium18 libsodium-dev pkg-config
WORKDIR /src/app
COPY . /src/app
RUN cargo install

CMD ["cargo run"]