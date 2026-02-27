FROM gcr.io/distroless/cc-debian12
ARG TARGETPLATFORM
COPY $TARGETPLATFORM/tinirun-server /usr/bin/tinirun-server
ENTRYPOINT ["/usr/bin/tinirun-server"]
