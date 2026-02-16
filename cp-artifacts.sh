#!/bin/sh

cp buildroot-2025.11/output/images/bcm2837-rpi-3-b.dtb bcm2837-rpi-3-b.dtb
cp buildroot-2025.11/output/images/Image.gz vmlinuz-tema2
cp buildroot-2025.11/output/images/rootfs.ext4 tema2.img
qemu-img resize tema2.img 16M

