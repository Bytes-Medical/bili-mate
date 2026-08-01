# Bili Mate API container (spec 10 container contract).
#
# The image contains only the API binary (with the approved rule pack
# embedded and self-tested at startup), CA certificates from the distroless
# base, and the required notices. It runs as a fixed unprivileged user and
# must be deployed with a read-only root filesystem and a bounded
# memory-backed /tmp (see infrastructure/ ECS task definition).
#
# Build:
#   docker build \
#     --build-arg SOURCE_COMMIT=$(git rev-parse HEAD) \
#     --build-arg VERSION=0.1.0 \
#     -t bili-mate-api .

# syntax=docker/dockerfile:1

FROM rust:1.90.0-slim-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
COPY apps ./apps
COPY spec ./spec
COPY NOTICE ./NOTICE
RUN cargo build --release --locked -p bili-mate-api

FROM gcr.io/distroless/cc-debian12:nonroot

ARG SOURCE_COMMIT=unknown
ARG VERSION=0.1.0
LABEL org.opencontainers.image.title="bili-mate-api" \
      org.opencontainers.image.description="Deterministic NICE CG98 neonatal jaundice decision-support API (professional use, UK only)" \
      org.opencontainers.image.source="https://github.com/bili-mate/bili-mate" \
      org.opencontainers.image.revision="${SOURCE_COMMIT}" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.licenses="UNLICENSED"

COPY --from=build /src/target/release/bili-mate-api /app/bili-mate-api
COPY --from=build /src/NOTICE /app/NOTICE
# Audit transparency: the normative rule-pack source shipped alongside the
# binary that embeds it (the binary verifies its own embedded copy at start).
COPY --from=build /src/spec/clinical/nice-cg98-2023-10-31.1.yaml /app/notices/rule-pack-source.yaml

# Distroless "nonroot": fixed unprivileged UID/GID 65532.
USER 65532:65532
ENV BILI_MATE_BIND=0.0.0.0:8080
EXPOSE 8080
ENTRYPOINT ["/app/bili-mate-api"]
