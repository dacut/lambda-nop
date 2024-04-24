FROM amazonlinux:2023 as builder
COPY ./target/aarch64-unknown-linux-gnu/release/bootstrap /bootstrap.aarch64
COPY ./target/x86_64-unknown-linux-gnu/release/bootstrap /bootstrap.x86_64
RUN cp /bootstrap.$(uname -m) /bootstrap

FROM public.ecr.aws/lambda/provided:al2023 as runtime
COPY --from=builder /builder/lambda-nop/target/release/bootstrap /var/task/bootstrap
