FROM rust:latest

WORKDIR /usr/src/beholder
COPY . .

RUN cargo install --path .

ENTRYPOINT ["beholder"]
