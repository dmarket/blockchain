FROM ubuntu:16.04

ARG CURRENT_NODE="node0"
ENV CURRENT_NODE ${CURRENT_NODE}

#RUN apt-get update && apt-get -y install libsodium18 libsodium-dev pkg-config
WORKDIR /src/app
COPY ./target/debug/dmbc-node /src/app/
RUN chmod +x /src/app/dmbc-node

CMD ["/src/app/dmbc-node"]
