# Controlul LED-urilor / GPIO

## Descriere
Aplicatie pentru controlul a 20 de pini GPIO, 
pentru animatii LED si gestionare asincrona.

### Configurare Hardware si Mapare Pini
LED-urile sunt conectate in configuratie **Katod Comun** (Active HIGH), dupa logica:
- `1` (HIGH) = **Aprins**
- `0` (LOW) = **Stins**

#### Mapare Pini
* **Pinii 0 - 17:** 6 LED-uri RGB (grupate in calupuri de cate 3: R, G, B).
  * *Exemplu:* Pin (0,1,0) -> R=0, G=1, B=0 inseamna doar **Verde aprins**
* **Pin 18:** LED Portocaliu (non-RGB)
* **Pin 19:** LED Mov (non-RGB)

### Logica de Control
Serviciul ruleaza un server HTTP [axum](https://docs.rs/axum/latest/axum/)
pe `localhost` care modifica o stare interna 
(array de enum-uri). Controlul efectiv al pinilor se face prin doua bucle asincrone independente:
1. **Loop LED-uri RGB:** Gestioneaza culorile si animatiile
2. **Loop LED-uri non-RGB:** Gestioneaza cele LED-urile care sunt legate de ultimii 2 pini GPIO

### Consistenta Animatiilor

Pentru a asigura un aspect vizual continuu, starea animatiilor nu se modifica instant la
primirea request-ului HTTP, ci este sincronizata la un interval de **1 secunda**
(durata unui cadru din animatie - la fiecare secunda, culorile LED-urilor RGB se vor schimba).


Fiecare animatie ruleaza propria rutina intr-un `for`
Daca am schimba animatia instant in momentul request-ului, bucla asincrona
(care  poate fi intr-un `sleep` in acel moment)
ar detecta schimbarea de abia dupa terminarea `sleep`-ului si ar apela din nou rutina,
resetand inca o data animatia (ceea ce nu este deloc placut).

Drept solutie, pentru a evita un dublu reset,
modificarile logice sunt procesate doar in interiorul buclei asincrone de control, la un interval fix de **1 secunda**.



## Endpoint-uri HTTP

> - [./api/postman-GPIO-control.json](./api/postman-GPIO-control.json)
> - [./api/restfox-GPIO-control.json](./api/restfox-GPIO-control.json)

### Health Check

| Metoda | Endpoint | Body (JSON) | Raspuns (JSON) | Descriere |
| :---: | :---: | :---: |:---:|:---:|
| GET | `api/health-check` | - | - | healt check |


### LED states (Helper Method)

| Metoda | Endpoint | Body (JSON) | Raspuns (JSON) | Descriere |
| :---: | :---: | :---: |:---:|:---:|
| GET | `api/possible-led-states` | - | `{ "REG LED": [...], "non-RGB LED": [...] }` | Culorile/starile posibile pe care le poate avea un LED |


### LED-uri RGB

| Metoda | Endpoint | Body (JSON) | Raspuns (JSON) | Descriere |
| :---: | :---: | :---: |:---:|:---:|
| GET | `/api/rgb-led/{id}` | - | `{ state: "white"/"red"/"blue"/"green"/"off" }` | Returneaza starea unui LED RGB |
| GET | `/api/rgb-led/{id}` | - | `{ 0: "blue", 1: "red", ... 5: "blue" }` | Returneaza un array cu starea tuturor LED-urilor RGB |


### LED-uri non-RGB

| Metoda | Endpoint | Body (JSON) | Raspuns (JSON) | Descriere |
| :---: | :---: | :---: |:---:|:---:|
| POST | `/api/non-rgb-led` | `{ id: 1..2, state: "on"/"off"/"blinking" }` | - | Seteaza starea unui LED non-RGB |
| GET | `/api/non-rgb-led/{id}` | - | `{ state: "on"/"off"/"blinking-on"/"blinking-off" }` | Returneaza starea unui LED non-RGB |
| GET | `/api/non-rgb-leds` | - | `{ 0: "on", 1: "blinking-on" }` | Returneaza un array cu starea tuturor LED-urilor non-RGB |


### Animatii

| Metoda | Endpoint | Body (JSON) | Raspuns (JSON) | Descriere |
| :---: | :---: | :---: |:---:|:---:|
| GET | `/api/current-animation` | - | `{ id: 1..., name: "Nume animatie" }` | Returneaza animatia curenta |
| POST | `/api/current-animation/{id}` | - | - | Porneste animatia cu ID-ul precizat in URL |
| POST | `/api/next-animation` | - | - | Trece la animatia urmatoare |
| POST | `/api/prev-animation` | - | - | Trece la animatia anterioara |
| GET | `/api/animations` | - | `{ 0: "blink", 1: "wave", ..., 6: "top-down"}` | Returneaza un array cu toate animatiile |



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
ls target/aarch64-unknown-linux-musl/release/gpio-control
```
