FROM ubuntu:20.04

# Update linux dist
RUN apt update

RUN apt install -y \
		curl \
		git \
		vim \
		build-essential

# Install Rust on container
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup toolchain install 1.31

# Copy project to container
COPY ./ /src/project

WORKDIR /src/project
