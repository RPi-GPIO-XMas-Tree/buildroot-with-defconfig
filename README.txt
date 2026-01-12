Readme Tema 2 SI 2025-2026
-------------------------------------------------------------------------------
Bogdan-Cristian Trifan, 344C2
-------------------------------------------------------------------------------

# Rezultate
tema2img       45M
vmlinuz-tema2  8.8M

Am folosit `musl` in loc de `glibc` si am dezactivat cat de multe drivere am putut.

# Setup Initial

Am inceput prin a descarca buildroot cea mai recenta versiune stabila de buildroot (2025.11)
am creat link simbolic pentru a scrie mai putin, si am ales 'raspberrypi3_64_defconfig'.
Conform https://gitlab.com/buildroot.org/buildroot/, 'make list-defconfigs' listeaza configuratiile posibile.


# make menuconfig

In menuconfig, am setat arhitectura aarch64, am instalat libgpiod si openssh (pe care l-am inlocuit mai tarziu cu dropbear),
am configurat hostname-ul, parola, am verificat ca apare 'eth0' la DHCP,
am creat un folder (separat de buildroot) pe care l-am folosit pentru overlay
(include configuratia SSH, binarele pentru cele 2 servicii si script-ul de init.d)
si am configurat consola care apare la bootare ('ttyS0' cu baudrate=115200 - aici sunt probleme).

Am configurat parametrii initiali ai kernel-ului pe care ii voi folosi in imagine:
- Am suprascrs linkul catre kernelul downstream cu cel mainline,
    torvalds/linux, versiunea v6.12, folosind link de tarball catre repo-ul de pe git (https://github.com/torvalds/linux/archive/refs/tags/v6.12.tar.gz)
- DTB-ul (broadcom/bcm2837-rpi-3-b)


# make linux-menuconfig

Pentru a rezolva problema de hash, am descarcat separat arhiva de linux v6.12 si am scris hash-ul ei (sha256sum) in board/raspberrypi/patches/linux/linux.hash.

Odata deschis meniul:
- am dezactivat modulele (kernel monolit)
- am adaugat numele meu in versiune
- am dezactivat `IP: kernel autoconfiguration` pentru a evita conflictele DHCP la boot


# Compilarea (make)

Compilarea a reusit dupa ce am marit partitia de boot la 128M in buildroot/board/raspberrypi3-64/genimage.cfg.in.

# Prima rulare functionala in QEMU
Dupa compilare (kernel, rootfs etc), am copiat
- kernelul (buildroot/output/images/Image)
- DTB-ul (buildroot/output/images/bcm2837-rpi-3-b.dtb)
- Imaginea de card SD (buildroot/output/images/sdcard.img) si am redimensionat-o puterea a lui 2

Am activat ulterior 'PermitRootLogin' in overlay-ul pentru SSH, pentru a permite conexiune SSH (initial era blocata).

# Aplicatii

Am lasata cate un README mai detaliat pentru cele 2 servicii care ruleaza in imagine:
1. GPIO Control (axum+rpall) -> expune un webserver HTTP (asculta local) catre WEB UI, logica de animatii/bit toggling
2. WEB UI (axum+reqwest) -> continous polling la fiecare secunda pentru a updata starea LED-urilor, proxy HTTP pentru a comunica cu GPIO Control

Ambele servicii au fost scrise in Rust si cross-compilate cu `aarch64-unknown-linux-musl` toolchain,
rezultand in cate un executabil, pe care le-am copiat in `overlay/bin` (vezi `build-apps.sh`).
Astfel, nu am mai avut nevoie de niciun interpretor (python/nodejs), economisind mult spatiu pe disc.

Pentru a vizualiza WEB UI, nu folosi un browser Chrome. Nu se vede bine.
Recomand Firefox/Brave.

Intrucat niciun serviciu nu depinde de `glic`, am putut face o optimizare.

# Optimizari
- Am dezactivat drivere/file-system-uri nefolosite din kernel (fiecare recompilare ~10-15 min.):
    Le-am dezactivat in calupuri, chiar daca cate unul ar fi fost mai safe, ar fi durat mult.
    Si utlerior am micsorat dimensiunea din genimage.cfg.in (nu trebuie sa fie putere a lui 2).
- Am incercat sa dezactivez "ce cred ca nu folosesc" din busybox (a fost un fiasco, a trebuit sa fac rollback)
- Am trecut de la `glibc` la `musl` (foarte costisitor la timp, a trebuit sa o iau de la 0, dar am pastrat config-ul de kernel)
    spatiul ocupat de `/dev/root` scazand de la ~23M la ~7.5M.
    Acest back-up de configuratie de kernel m-a salvat pe mine de la a dezactiva driverele nefolosite pentru a o a doua oara.
- Am aplicat compresie asupra imaginii de kernel ('Image.gz' Kernel binary + 'xz compression' + 'optimize for size (-0s)')
- Am inlocuit `openssh` cu `dropbear`
- Am eliminat libgpiod (rpall se descurca si fara)


# Problemute
Desi SSH-ul si interfata WEB merg,
consola deschisa de script-ul de lansare are urmatoarele probleme:
- Daca setez getty la `ttyAMA0` (din menuconfig) -> script-ul scrie 'can't open /dev/ttyAMA0: No such file or directory'
- Daca setez la `ttyS0` -> nu se afiseaza nimic legat de linia de consola
- Daca setez la `console` -> apare un prompt, dar pare ca nu imi ia input-ul

Nu am prea inteles ce face Makefile-ul, asa ca mi-am scris un script de arhivare.
