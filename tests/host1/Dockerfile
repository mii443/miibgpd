FROM rust:1-buster

WORKDIR /miibgpd

RUN rustup default nightly

COPY . .
RUN cargo build

CMD ["./target/debug/miibgpd", "64512 10.200.100.2 64513 10.200.100.3 active"]
