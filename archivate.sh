#!/bin/sh

BIN_ARCHIVE="SI-Tema2-bin.tar.xz"
SRC_ARCHIVE="SI-Tema2-src.zip"

rm -f $BIN_ARCHIVE $SRC_ARCHIVE

tar cJf $BIN_ARCHIVE \
    bcm2837-rpi-3-b.dtb \
    vmlinuz-tema2 \
    tema2.img  \
    launch-tema2.sh

sha256sum $BIN_ARCHIVE > checksum.txt


cp buildroot-2025.11/.config buildroot_config
cp buildroot-2025.11/package/busybox/busybox.config busybox_config

zip -r $SRC_ARCHIVE \
    checksum.txt \
    apps-rpi3/* \
    README.txt \
    url.txt \
    overlay-tema2025/ \
    build-apps.sh \
    buildroot_config \
    kernel_config \
	busybox_config \
    archivate.sh \
    cp-artifacts.sh \
    -x "**/target/*" # Exclude executabilele compilate de cargo



if [ -d $HOME/Downloads ] ; then
    rm -f $HOME/Downloads/$BIN_ARCHIVE
    rm -f $HOME/Downloads/$SRC_ARCHIVE
    cp ./$BIN_ARCHIVE ./$SRC_ARCHIVE $HOME/Downloads/
fi
