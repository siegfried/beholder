FROM rust:1.57

WORKDIR /usr/src/beholder
COPY . .

RUN cargo install --path .

ENTRYPOINT ["beholder"]
