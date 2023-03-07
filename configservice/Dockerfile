FROM rust:latest AS builder

RUN update-ca-certificates

# Create appuser
ENV USER=demo
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /demo

ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

COPY ./ .

RUN cargo build --release

## Final image
FROM gcr.io/distroless/cc

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /demo

# Copy our build
COPY --from=builder /demo/target/release/config-manager ./

RUN ls -l
# Use an unprivileged user.
USER demo:demo

CMD ["/demo/config-manager"]