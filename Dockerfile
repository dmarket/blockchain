FROM ubuntu:bionic

ARG TYPE=debug

WORKDIR /src/app
COPY ./target/$TYPE/dmbc-node /src/app/
COPY ./target/$TYPE/dmbc-discovery /src/app/
RUN mkdir /src/app/etc
COPY ./etc/config.toml /src/app/etc/config.toml

RUN chmod +x /src/app/dmbc-node
RUN chmod +x /src/app/dmbc-discovery

CMD ["/src/app/dmbc-node"]
