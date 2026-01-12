# Makefile pentru Tema 2, 2025-2026 v0.1
# Modificați denumiri de fișiere sursă unde este cazul

# Fișiere de inclus în arhiva binară
DTB_FILENAME = bcm2837-rpi-3-b.dtb
BINARY_ARCHIVE = bin_archive.tar.xz
BINARY_FILES = tema2.img vmlinuz-tema2 $(DTB_FILENAME) \
			   launch-tema2.sh

# TODO: adăugați fișiere / patternuri de ignorat (orice e binar / ocupă mult!)
SOURCE_IGNORE_FILES = README.skel.txt README.notes.md \
					  buildroot-* linux-* \
					  vmlinuz* *.img *.dtb $(BINARY_FILES) \
					  *.zip *.tar.* build/* .git target/

PYTHON3 = /usr/bin/python3

SHELL=/bin/bash
all: bin_archive checksum source_archive

A?=
run:
	./launch-tema2.sh $(A)

gpio_viewer:
	$(PYTHON3) gpio-viewer.py $(A)

# generates the binary archive
bin_archive: $(BINARY_ARCHIVE)
$(BINARY_ARCHIVE): $(BINARY_FILES)
	tar cJvf "$@" $^
	ls -lh "$@"

# genereates the checksum of the current binary archive
checksum: checksum.txt
checksum.txt: $(BINARY_ARCHIVE)
	sha256sum "$(BINARY_ARCHIVE)" > checksum.txt
	cat checksum.txt

source_archive: source_archive.zip
source_archive.zip: checksum.txt
	@if ! command -v zip &>/dev/null; then echo "Please install zip!"; exit 1; fi
	@if [ ! -f checksum.txt ]; then echo "No checksum.txt present!"; exit 1; fi
	@rm -f "$@"
	zip -r -y "$@" . \
		$(patsubst %,-x '%',$(SOURCE_IGNORE_FILES))
	ls -lh "$@"

clean:
	rm -rf "$(BINARY_ARCHIVE)" source_archive.zip

.PHONY: build bin_archive checksum source_archive clean

# Python3 VirtualEnv creation (disabled, no external deps required)
$(PYTHON3):
	python3 -mvenv .venv
	$(PYTHON3) -mpip install -r requirements.txt

# Decomentați pentru a activa regulile de mai jos (SUNT EXEMPLE DE COPIERE
# kernel + imagine + DTB din directorul vostru de build!)
#COPY_FILES_LINUX=1
#COPY_FILES_BUILDROOT=1

ifeq ($(COPY_FILES_LINUX),1)
$(DTB_FILENAME): $(wildcard linux-*)/arch/arm64/boot/dts/$(DTB_FILENAME)
	cp -f "$<" "$@"
vmlinuz-tema2: $(wildcard linux-*)/arch/arm64/boot/Image.gz
	cp -f "$<" "$@"
endif

ifeq ($(COPY_FILES_BUILDROOT),1)
tema2.img: $(wildcard buildroot-*)/output/images/rootfs.ext4
	cp -f "$<" "$@"
	# e.g.: resize to power of two (TODO: !!CHANGE THIS!!)
	qemu-img resize "$@" '64M'
endif

