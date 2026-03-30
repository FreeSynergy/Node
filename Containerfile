FROM fedora:41

WORKDIR /app

# Copy pre-built binary (built by CI)
COPY cli/target/release/fs-node-server /usr/local/bin/fs-node-server
COPY cli/target/release/fs-node-cli    /usr/local/bin/fsn

# Runtime user
RUN useradd -r -s /sbin/nologin fsnode

USER fsnode

EXPOSE 8080 9090

ENTRYPOINT ["/usr/local/bin/fs-node-server"]

LABEL org.opencontainers.image.source="https://github.com/FreeSynergy/fs-node"
LABEL org.opencontainers.image.description="FreeSynergy Node — deployment engine and API"
