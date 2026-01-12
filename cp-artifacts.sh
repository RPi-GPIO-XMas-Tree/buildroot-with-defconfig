#!/bin/sh

cp buildroot/output/images/Image.gz vmlinuz-tema2
cp buildroot/output/images/bcm2837-rpi-3-b.dtb bcm2837-rpi-3-b.dtb
cp buildroot/output/images/rootfs.ext4 tema2.img
qemu-img resize tema2.img 64M


