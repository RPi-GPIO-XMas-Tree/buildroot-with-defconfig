# Controlul LED-urilor / GPIO

## Endpoint-uri HTTP


### Health Check

| Metodă | Endpoint | Body (JSON) | Răspuns (JSON) | Descriere |
| :---: | :---: | :---: |:---:|:---:|
| GET | `api/health-check` | - | - | healt check |


### LED states (Helper Method)

| Metodă | Endpoint | Body (JSON) | Răspuns (JSON) | Descriere |
| :---: | :---: | :---: |:---:|:---:|
| GET | `api/possible-led-states` | - | `{ "REG LED": [...], "non-RGB LED": [...] }` | Culorile/starile posibile pe care le poate avea un LED |


### LED-uri RGB

| Metodă | Endpoint | Body (JSON) | Răspuns (JSON) | Descriere |
| :---: | :---: | :---: |:---:|:---:|
| GET | `/api/rgb-led/{id}` | - | `{ state: "white"/"red"/"blue"/"green"/"off" }` | Returnează starea unui LED RGB |
| GET | `/api/rgb-led/{id}` | - | `{ 0: "blue", 1: "red", ... 5: "blue" }` | Returneaza un array cu starea tuturor LED-urilor RGB |


### LED-uri non-RGB

| Metodă | Endpoint | Body (JSON) | Răspuns (JSON) | Descriere |
| :---: | :---: | :---: |:---:|:---:|
| POST | `/api/non-rgb-led` | `{ id: 1..2, state: "on"/"off"/"blinking" }` | - | Setează starea unui LED non-RGB |
| GET | `/api/non-rgb-led/{id}` | - | `{ state: "on"/"off"/"blinking-on"/"blinking-off" }` | Returnează starea unui LED non-RGB |
| GET | `/api/non-rgb-leds` | - | `{ 0: "on", 1: "blinking-on" }` | Returneaza un array cu starea tuturor LED-urilor non-RGB |


### Animatii

| Metodă | Endpoint | Body (JSON) | Răspuns (JSON) | Descriere |
| :---: | :---: | :---: |:---:|:---:|
| GET | `/api/current-animation` | - | `{ id: 1..., name: "Nume animatie" }` | Returnează animația curentă |
| POST | `/api/current-animation/{id}` | - | - | Pornește animația cu ID-ul precizat in URL |
| POST | `/api/next-animation` | - | - | Trece la animația următoare |
| POST | `/api/prev-animation` | - | - | Trece la animația anterioară |
| GET | `/api/animations` | - | `{ 0: "blink", 1: "wave", ..., 6: "top-down"}` | Returneaza un array cu toate animatiile |



## Compilare

```sh
$ cargo build
```
