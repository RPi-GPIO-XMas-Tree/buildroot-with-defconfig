Readme Tema 2 SI 2025-2026
-------------------------------------------------------------------------------
Bogdan-Cristian Trifan, 344C2
-------------------------------------------------------------------------------

# Rezultate
tema2.img       8M
vmlinuz-tema2  5.6M

Eu am facut initial tema cu 'raspberrypi3_64_defconfig', dar dupa problemele de consola, am luat-o (aproape) de la capat cu un defconfig generic.
Daca pe defconfig-ul de la rpi ma chinuiam sa duc imaginea de cardSD sub 64M, pe cel generic am dus-o la 8M fara probleme. Chiar si imaginea finala de kernel a fost mult mai mica.

Am rezolvat tema odata ce am inteles artefactele pe care le genereaza 'defconfig'-ul generic in urma compilarii. Cel generic nu foloseste o imagine partitionata (am modificat si script-ul de launch), in timp ce acela de rpi3 da.


Am folosit 'musl' in loc de 'glibc' si am dezactivat cat de mult am putut din kernel.


# Setup Initial

Am inceput prin a descarca cea mai recenta versiune stabila de buildroot (2025.11).
De data asta, am ales defconfig-ul generic ('make defconfig').

# make menuconfig

In menuconfig, am setat arhitectura 'aarch64 (little endian)', varianta 'cortex-A53',
am instalat libgpiod (l-am dezactivat ulterior) si openssh (l-am inlocuit mai tarziu cu dropbear),
am configurat hostname-ul, parola, am trecut 'eth0' la interfata configurata prin DHCP,
am creat un folder (separat de buildroot) pe care l-am folosit pentru overlay
(include configuratia SSH, binarele pentru cele 2 servicii si script-ul din init.d cu permisiuni de executie)
si am configurat consola care apare in QEMU la bootare ('ttyS1' cu baudrate=115200).

Am selectat explicit ext4 la imaginea filesystem-ului, si i-am micsorat dimensiunea dupa fiecare optimizare a kernelului.

Am configurat parametrii initiali ai kernel-ului:
- URL: https://github.com/torvalds/linux/archive/refs/tags/v6.12.tar.gz 
- DTB-ul: broadcom/bcm2837-rpi-3-b
- Inainte sa fac dezactivarile de kernel, am copiat fisierul in afara buildroot-ului (ca backup)
    si am specificat calea catre el ('using a custom (def)config file')

Si am activat firmware-ul default pentru rpi 0/1/2/3 (target pckgs -> hw handling).

# make linux-menuconfig

Pentru a rezolva problema de hash, am descarcat separat arhiva de linux v6.12 si am scris hash-ul ei (sha256sum) in 'buildroot-2025.11/linux/linux.hash'.

Odata deschis meniul:
- am dezactivat suportul pentru modulele (kernel monolit)
- am adaugat numele meu in versiune si am dezactivat 'auto append version info....'
- am scris numele default hostname-ului
- am verificat ca este activat 'IP: kernel autoconfiguration' -> 'IP: DHCP support' din optiunile de retea


# Compilarea (make)

Am folosit script-ul 'cp-artifacts.sh' pentru a copia fisierele generate de buildroot in urma compilarii.

Am activat ulterior 'PermitRootLogin' in overlay-ul pentru SSH, pentru a permite conexiune SSH (initial era blocata).

# 🦀 Aplicatii

Am lasat cate un README mai detaliat pentru cele 2 servicii care ruleaza in imagine:
1. GPIO Control (axum+rpall) -> expune un webserver HTTP (asculta local) catre WEB UI, logica de animatii/bit toggling
2. WEB UI (axum+reqwest) -> continous polling la fiecare secunda pentru a updata starea LED-urilor, proxy HTTP pentru a comunica cu GPIO Control

Ambele servicii au fost scrise in Rust si cross-compilate cu `aarch64-unknown-linux-musl` toolchain,
rezultand in cate un executabil, pe care le-am copiat in `overlay/bin` (cu `build-apps.sh`).
Astfel, nu am mai avut nevoie de niciun interpretor (python/nodejs), economisind mult spatiu pe disc.

Cele doua servicii comunica local, folosind endpoint-uri HTTP.

Pentru a vizualiza WEB UI, nu folosi un browser Chrome. Nu se vede bine. Nici Brave.
Recomand Firefox.


Pentru scriptul de init, am folosit o prioritate mai mare decat cea a serviciilor deja existente.

Primii 18 pini sunt grupati cate 3 (6 LED-uri RGB); iar de utimii 2 pini sunt legati cate un LED non-RGB (culoare predefinita in WEB UI).
Logica este de active pe HIGH, iar LED-urile RGB pot avea si alte culori decat cele de baza, prin imbinarea a 2 canale RGB.


Intrucat niciun serviciu nu depinde de 'glic', am putut face o optimizare.


# Optimizari
- Am inlocuit toolchain-ul 'glibc' cu 'musl'
    foarte costisitor la timp, a trebuit sa o iau de la 0, dar am pastrat config-ul de kernel
    spatiul ocupat de '/dev/root' a scazut de la (peste) 20M la sub 10M.
- Am aplicat compresie asupra imaginii de kernel de menuconfig ('Image.gz' kernel binary format + 'xz' kernel compression format)
- Am ales flag-ul '-0s' (optimise for size) din configuratia generala de kernel 
- Am inlocuit 'openssh' cu 'dropbear'
- Am eliminat 'libgpiod' (rppal se descurca si fara)
- Am dezactivat la greu tot ce am putut din kernel (drivere, file system-uri, kernel hacking, platforme - mai putin broadcomm)
- Am comprimat rootfs-ul cu xz



# Arhivare
Nu am prea inteles ce face Makefile-ul, asa ca mi-am scris un script de arhivare.


# Bonus (optional 🙂)
Daca solutia mea ajunge in top 10 cu dimensiunea imaginii
si ti se pare ca aspectul/functionalitatea chiar ies din tipar,
promit ca nu refuz un pin (non-GPIO) pentru laptop 😄

Indiferent de asta, multumesc pentru corectare!



# Packete suplimentare

Pentru ca sistemul sa poate fi testat prin teste automate, a fost nevoie de:
1. Datetime UTC curent (nu anul nou 1970):
	- in busybox am activat `ntpd`
	- am creat un script de init.d care sa obtina automat la boot data curenta
2. Utilitarul `curl`: am cautat dupa `libcurl` in menuconfig si am activat optiunea
3. DNS resolution: fisierul `/etc/resolv.conf` foloseste 2 servere DNS
4. Dimensiunea imaginii a trebuit marita putin

Pentru usurinta, am copiat fisierul de configuratie busybox (packages/busybox/busybox.config)
intr-un fisier din afara buildroot-ului, si l-am linkat in menuconfig.

In urma activarii serviciului NTPd din busybox, am rulat `make busybox-rebuild` inainte de `make`.
PS: `make busysbox-reconfigure` revine la setarile default (cred).



