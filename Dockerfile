FROM rust:1.61

WORKDIR /usr/src/beholder
COPY . .

RUN cargo install --path .

ENTRYPOINT ["beholder"]
