# WEB UI

## Descriere

Interfata web construita cu HTML/CSS/JS.
Pentru backend, am folosit initial `NodeJS`,
dar mi-am dat seama ca este o optimizare daca il rescriu in Rust
(salvand spatiu pe disc).


### Configuratie LED-uri
Structura fizica a LED-urilor in brad:
- LED-uri RGB:
    - `idx 0`: Varf (Stea)
    - `idx 1-2`: Randul din mijloc (stanga -> dreapta)
    - `idx 3-5`: Randul de jos (stanga -> dreapta)
- LED-uri non-RGB:
    - `Portocaliu`: Extrema stanga-jos
    - `Mov`: Extrema dreapta-jos

### Comunicare GPIO Control
WEB UI interactioneaza cu serviciul **GPIO Control** (server Axum) prin cereri HTTP (GET/POST).
Aceasta este metoda fireasca de **IPC** pentru o aplicatie web.

Butoanele declanseaza metode **POST** pentru a schimba starea LED-urilor/animatia.

Pentru sincronizare, starea LED-urilor este actualizata prin **continuous polling**,
trimitand un GET request la fiecare 1 secunda.


### Proxy HTTP
Daca interfata mergea bine inainte sa o incarc in imagine,
in embedded lucrurile se complica un pic:
request-urile catre `localhost/api` (despre care credeam ca reprezinta comunicatia cu serviciul de GPIO Control),
de fapt ajungeau la mine pe calculator.
In consola browser-ului, am observat erori `CORS fetch` la liniile din JS care faceau cererile API.


Pentru a rezolva problema, am implementat backend-ul interfetei WEB folosind un Proxy HTTP,
care sa faca legatura intre browser-ul (din afara QEMU) si comunicatia **IPC** cu celalalt servicu.



### IP API
Pentru a arata in WEB UI ca imaginea construita chiar are acces la internet,
am facut un request in afara calculatorului, catre IP API (de acolo iau adresa publica si orasul).

## Probleme cu browser-ul
Interfata web NU ruleaza deloc bine in Chrome.

Ca alternativa, recomand Firefox/Brave.



## Cross-compiling

### Adaugare dependinte de compilare

```sh
sudo apt update
sudo apt install gcc-aarch64-linux-gnu -y

rustup target add aarch64-unknown-linux-musl
```

Am adaugat in `.cargo/config.toml`

```toml
[target.aarch64-unknown-linux-musl]
linker = "aarch64-linux-gnu-gcc"
```


### Compilare efectiva

> Nu uita de **flag**-ul `release`. Face foarte multe **optimizari** pentru hardware.

```sh
cargo build --target aarch64-unknown-linux-musl --release
```


### Locatia executabilelor generate

```sh
ls target/aarch64-unknown-linux-musl/release/web-ui
``
