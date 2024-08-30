FROM rust:1.80-bullseye

WORKDIR /app

COPY ./ ./

RUN cargo build --release

EXPOSE 7878

ENV DATABASE_URL=./dev.sqlite

CMD ["./target/release/cuddy_supervisor"]