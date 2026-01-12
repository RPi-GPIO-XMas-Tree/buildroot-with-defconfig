# Tema SI - raspberrypi3_64_defconfig

- buildroot-2025.11 (stable)
- linux kernel: v6.12

make defconfig

## make menuconfig

- In `Target options`, am setat arhitectura la `AArch64 (little endian)` si varianta la `cortex-A53`

- Am configurat hostname (*tema2025*), parola (*tema2025*) si folderul de overlay (*../overlay-tema2025*) in `System configuration`
- Am activat `dropbear` in aplicatiile de retea 
- `Kernel` -> Am bifat `[*] Linux Kernel`
- `Kernel version` -> `Custom tarball`
- `URL of custom kernel tarball` -> `https://github.com/torvalds/linux/archive/refs/tags/v6.12.tar.gz`
- Am descarcat local arhiva si am pus HASH-ul ei in fisierul `linux/linux.hash`: `sha256  c148131ddf77ec5252eb1067cad8ed62b498c4bee79cc46b44659f2348cbd494  v6.12.tar.gz`
- `Kernel` -> Am bifat `Build a Device Tree Blob (DTB)`
    -> Am completat `In-tree Device Tree Source file names` cu *broadcom/bcm2837-rpi-3-b*
- `Kernel` -> `Kernel configuration` -> am selectat `Use the architecture default configuration`
- `Host utilites` -> am activat `genimage`
- `Target packeges` -> `Hardware handling` -> `firmware` -> Am bifat:
  - `[*] rpi-firmware`
  - `[*] rpi 0/1/2/3 (default)` 
- `Fileystems images` -> am bifat `ext2/3/4 root filesystem`
  - am selectat `ext4` 


```sh
% cat linux/linux.hash                
sha256  c148131ddf77ec5252eb1067cad8ed62b498c4bee79cc46b44659f2348cbd494  v6.12.tar.gz
```

## make linux-menuconfig
- `General setup`
  - `Local version - append to kernel release`: *-si-bogdan.trifan*
  - Am debifat `[] Automatically append version info ...`
  - `Default hostname`: *tema2025*
- Am dezactivat `[] Enable loadable module support`
- `Network support` -> `Networking options` -> Am activat `[*] IP: kernel level autoconfiguration` -> `[*] IP: dchp`



## Copiere fisier configuratie kernel

```sh
cp buildroot/output/build/linux-custom/.config kernel_config
```


`make menuconfig` -> `Kernel configuration`
- `Use a custom (def)config file`
- `Configuration file path`: *../kernel_config*