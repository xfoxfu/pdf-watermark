FROM rust:1.78-bookworm as builder

RUN mkdir -p /app
WORKDIR /app

RUN mkdir -p /app/pdfium
WORKDIR /app/pdfium
RUN wget https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F6447/pdfium-linux-x64.tgz
RUN tar xzvf pdfium-linux-x64.tgz

WORKDIR /app
RUN cargo new --bin pdf-watermark .
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo build --release --frozen \
    && rm src/*.rs target/release/deps/pdf_watermark*

ADD . ./

RUN cargo build --release --frozen

FROM debian:bookworm-slim

RUN mkdir -p /app && \
    apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/pdf-watermark pdf-watermark
COPY --from=builder /app/pdfium/lib/libpdfium.so libpdfium.so

CMD ["/app/pdf-watermark"]
