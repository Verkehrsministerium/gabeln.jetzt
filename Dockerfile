FROM clux/muslrust:nightly
COPY . /volume
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM scratch
ENV SSL_CERT_FILE=/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/certs
COPY --from=0 /volume/target/x86_64-unknown-linux-musl/release/gabeln-jetzt /
COPY --from=0 /volume/assets /assets
COPY --from=0 /etc/ssl/certs /certs
ENTRYPOINT [ "/gabeln-jetzt" ]
