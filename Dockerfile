FROM --platform=$BUILDPLATFORM rust:1.78-bookworm as builder

ARG BUILDARCH
ARG TARGETPLATFORM
ARG TARGETARCH

RUN mkdir -p /app
WORKDIR /app

RUN case "${TARGETPLATFORM}" in \
    "linux/amd64") echo "export PDFIUM_PLATFORM=linux-x64" >> /app/.env; \
                   echo "export RUST_TARGET=x86_64-unknown-linux-gnu" >> /app/.env; \
                   echo "export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc" >> /app/.env ;; \
    "linux/arm64") echo "export PDFIUM_PLATFORM=linux-arm64" >> /app/.env; \
                   echo "export RUST_TARGET=aarch64-unknown-linux-gnu" >> /app/.env; \
                   echo "export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc" >> /app/.env ;; \
    *) echo "Unsupported platform ${TARGETPLATFORM}"; exit 1 ;; \
    esac && \
    cat /app/.env

RUN . /app/.env && \
    apt-get update && apt-get install -y crossbuild-essential-${TARGETARCH} && \
    rustup target add ${RUST_TARGET}

RUN mkdir -p /app/pdfium
WORKDIR /app/pdfium
RUN . /app/.env && \
    wget https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F6447/pdfium-${PDFIUM_PLATFORM}.tgz && \
    tar xzvf pdfium-${PDFIUM_PLATFORM}.tgz

WORKDIR /app
RUN cargo init --bin --name pdf-watermark
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN . /app/.env && \
    cargo build --release --locked --target ${RUST_TARGET} && \
    rm src/*.rs target/${RUST_TARGET}/release/deps/pdf_watermark*

ADD . ./

RUN . /app/.env && \
    cargo build --release --frozen --target ${RUST_TARGET} && \
    mkdir -p /app/target/release && \
    cp /app/target/${RUST_TARGET}/release/pdf-watermark /app/target/release/pdf-watermark && \
    cp /app/target/${RUST_TARGET}/release/mark_pdf /app/target/release/mark_pdf

FROM debian:bookworm-slim

RUN mkdir -p /app && \
    apt-get update \
    && apt-get install -y ca-certificates tzdata tini \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/pdf-watermark pdf-watermark
COPY --from=builder /app/target/release/mark_pdf mark_pdf
COPY --from=builder /app/pdfium/lib/libpdfium.so libpdfium.so

CMD ["/usr/bin/tini", "--", "/app/pdf-watermark"]
