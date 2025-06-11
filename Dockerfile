# https://github.com/kristof-mattei/rust-seed/blob/main/Dockerfile
FROM --platform=$BUILDPLATFORM rust:bookworm AS rust-base

ARG APPLICATION_NAME

RUN rm -f /etc/apt/apt.conf.d/docker-clean \
    && echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' >/etc/apt/apt.conf.d/keep-cache

RUN --mount=type=cache,id=apt-cache-amd64,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,id=apt-lib-amd64,target=/var/lib/apt,sharing=locked \
    apt-get update \
    && apt-get --no-install-recommends install --yes \
        build-essential \
        libsqlite3-dev

FROM rust-base AS rust-linux-amd64
ARG TARGET=x86_64-unknown-linux-gnu


RUN --mount=type=cache,id=apt-cache-amd64,from=rust-base,source=/var/cache/apt,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,id=apt-lib-amd64,from=rust-base,source=/var/lib/apt,target=/var/lib/apt,sharing=locked \
    dpkg --add-architecture amd64 \
    && apt-get update \
    && apt-get --no-install-recommends install --yes \
        libc6-dev-amd64-cross \
        gcc-x86-64-linux-gnu


FROM rust-base AS rust-linux-arm64
ARG TARGET=aarch64-unknown-linux-gnu

RUN --mount=type=cache,id=apt-cache-arm64,from=rust-base,source=/var/cache/apt,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,id=apt-lib-arm64,from=rust-base,source=/var/lib/apt,target=/var/lib/apt,sharing=locked \
    dpkg --add-architecture arm64 \
    && apt-get update \
    && apt-get --no-install-recommends install --yes \
        libc6-dev-arm64-cross \
        gcc-aarch64-linux-gnu

FROM rust-${TARGETPLATFORM//\//-} AS rust-cargo-build

ARG TARGETPLATFORM
ARG TARGETOS
ARG TARGETARCH

RUN echo "${TARGETPLATFORM} | ${TARGET} | ${TARGETOS} | ${TARGETARCH}"
RUN rustup target add ${TARGET}

WORKDIR /build
RUN cargo new ${APPLICATION_NAME}
WORKDIR /build/${APPLICATION_NAME}
COPY .cargo ./.cargo
COPY Cargo.toml Cargo.lock ./

RUN --mount=type=cache,target=/build/${APPLICATION_NAME}/target \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git/db,sharing=locked \
    --mount=type=cache,id=cargo-registry,target=/usr/local/cargo/registry/,sharing=locked \
    cargo build --release --target ${TARGET}

FROM rust-cargo-build AS rust-build

WORKDIR /build/${APPLICATION_NAME}

COPY src ./src

# ensure cargo picks up on the change
RUN touch ./src/main.rs

RUN --mount=type=cache,target=/build/${APPLICATION_NAME}/target \
    --mount=type=cache,id=cargo-git,target=/usr/local/cargo/git/db,sharing=locked \
    --mount=type=cache,id=cargo-registery,target=/usr/local/cargo/registry/,sharing=locked \
    cargo install --path . --target ${TARGET} --root /output


FROM alpine:3 AS passwd-build

RUN addgroup --gid 900 appgroup \
    && adduser --ingroup appgroup --uid 900 --system --shell /bin/false appuser

RUN cat /etc/group | grep appuser > /tmp/group_appuser
RUN cat /etc/passwd | grep appuser > /tmp/passwd_appuser

FROM gcr.io/distroless/cc-debian12

ARG APPLICATION_NAME

COPY --from=passwd-build /tmp/group_appuser /etc/group
COPY --from=passwd-build /tmp/passwd_appuser /etc/passwd

USER appuser

WORKDIR /app

COPY --from=rust-build --chown=appuser:appuser /output/bin/${APPLICATION_NAME} /app/entrypoint
COPY --chown=appuser:appuser static  /app/static
COPY --chown=appuser:appuser templates  /app/templates
COPY --chown=appuser:appuser config.toml  /app/config.toml

VOLUME /app/data
EXPOSE 1337

ENV RUST_BACKTRACE=full
ENTRYPOINT ["/app/entrypoint"]
