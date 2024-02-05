FROM archlinux:latest as build-stage

RUN pacman -Syyu --noconfirm --needed archlinux-keyring

# create a build user
RUN useradd -m builder && \
    echo "builder ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/builder

USER builder
WORKDIR stl2thumbnail_rs/dist/archlinux

RUN makepkg -cfs
