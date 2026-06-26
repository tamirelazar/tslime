# Self-contained build: rust:alpine targets x86_64-unknown-linux-musl by
# default, so a plain release build produces a static binary that runs on an
# empty `scratch` image. Final image is just the ~3.6MB binary — no OS layer.
FROM rust:alpine AS build
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY . .
RUN cargo build --release

FROM scratch
COPY --from=build /src/target/release/tslime /tslime
ENTRYPOINT ["/tslime"]
