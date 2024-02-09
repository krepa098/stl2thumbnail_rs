FROM archlinux:latest AS build-stage

# install required packages
RUN pacman -Syyu --noconfirm --needed archlinux-keyring sudo base-devel cmake extra-cmake-modules

# create a build user
RUN useradd -m builder && \
    echo "builder ALL=(ALL) NOPASSWD: ALL" > /etc/sudoers.d/builder

# create build directory
RUN mkdir -p /build

# copy the repo to the container
WORKDIR /build
COPY / ./stl2thumbnail_rs

# change owner of build folder
RUN chown -R builder /build

USER builder

# build stl2thumbnail-git
WORKDIR /build/stl2thumbnail_rs/dist/archlinux/stl2thumbnail-git
RUN makepkg -cfs --noconfirm

RUN mv stl2thumbnail-git-v*.pkg.tar.zst stl2thumbnail-git.pkg.tar.zst

# install 'stl2thumbnail-git' pkg required by 'stl2thumbnail-kde-git'
USER root
RUN pacman --noconfirm --needed -U stl2thumbnail-git.pkg.tar.zst
USER builder

# build stl2thumbnail-kde-git
WORKDIR /build/stl2thumbnail_rs/dist/archlinux/stl2thumbnail-kde-git
RUN makepkg -cfs --noconfirm

RUN mv stl2thumbnail-kde-git-v*.pkg.tar.zst stl2thumbnail-kde-git.pkg.tar.zst

# prepare files to be copied to host
FROM scratch AS export-stage
COPY --from=build-stage /build/stl2thumbnail_rs/dist/archlinux/stl2thumbnail-git/stl2thumbnail-git.pkg.tar.zst /
COPY --from=build-stage /build/stl2thumbnail_rs/dist/archlinux/stl2thumbnail-kde-git/stl2thumbnail-kde-git.pkg.tar.zst /