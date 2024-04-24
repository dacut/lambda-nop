FROM amazonlinux:2023 as builder
RUN dnf install -y clang util-linux-user
RUN useradd --system --home-dir /builder --shell /sbin/nologin builder
RUN mkdir /builder
RUN chown -R builder:builder /builder
USER builder
COPY rustup.sh /builder/
RUN /builder/rustup.sh -y
ENV PATH=/builder/.cargo/bin:$PATH
RUN mkdir /builder/lambda-nop
COPY Cargo.toml Cargo.lock /builder/lambda-nop/
COPY src/bin/bootstrap.rs /builder/lambda-nop/src/bin/
USER builder
WORKDIR /builder/lambda-nop
RUN cargo build --release

FROM scratch as executable
COPY --from=builder /builder/lambda-nop/target/release/bootstrap /bootstrap

FROM public.ecr.aws/lambda/provided:al2023 as runtime
COPY --from=builder /builder/lambda-nop/target/release/bootstrap /var/task/bootstrap
