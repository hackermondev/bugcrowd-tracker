FROM rust:1.87 as builder
WORKDIR /usr/src/bugcrowd_tracker

COPY . .
RUN cargo install --path bugcrowd_tracker

FROM ubuntu
COPY --from=builder /usr/local/cargo/bin/bugcrowd_tracker /usr/local/bin/bugcrowd_tracker
ENTRYPOINT ["/usr/local/bin/bugcrowd_tracker"]