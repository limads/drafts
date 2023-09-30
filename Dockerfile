FROM ubuntu:lunar

# TODO: Use Earthly instead and make me an Earthfile

ARG DEBIAN_FRONTEND=noninteractive

RUN apt update 

# General utilities ; build tools ; library dependencies for drafts.
RUN apt install --yes   curl                                                     \
                                                                                 \
                        clang build-essential                                    \
                                                                                 \
                        libpoppler-dev libpoppler-glib-dev                       \
                        libglib2.0-dev                                           \
                        libcairo-dev                                             \
                        libgtk-4-dev libgtksourceview-5-dev libadwaita-1-dev     


# Install rust. HAs to be done slightly differently for a container, to install in the PATH.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain stable -y

# Add utilities to path - sourcing as the installer script details won't work easily with Dockerfile
ENV PATH=/root/.cargo/bin:$PATH


# Check out source
COPY     . /drafts/
WORKDIR    /drafts/

# Build it
RUN cargo install --path .

# Find drafts in /root/.cargo/bin/
