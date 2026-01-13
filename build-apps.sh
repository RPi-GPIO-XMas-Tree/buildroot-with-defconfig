#!/bin/sh

cd apps-rpi3/gpio-control \
    && cargo build --target aarch64-unknown-linux-musl --release \
    && cd - \
    && cp apps-rpi3/gpio-control/target/aarch64-unknown-linux-musl/release/gpio-control overlay-tema2025/bin/


cd apps-rpi3/web-ui \
    && cargo build --target aarch64-unknown-linux-musl --release \
    && cd - \
    && cp apps-rpi3/web-ui/target/aarch64-unknown-linux-musl/release/web-ui overlay-tema2025/bin/

chmod +x overlay-tema2025/etc/init.d/S99tema2
