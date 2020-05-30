Nayok
=
Nayok is server that allows to receive notifications and replay them later.
It has 2 endpoints:
1. `/notifications`: saves request into local sqlite db giving back "OK" 
with http code 200 to the caller. 
2. `/notification-results`: returns list of received notifications in json format  


Build production
=
```shell script
cargo build --target=x86_64-unknown-linux-musl --bin=nayok --release
```     
Manual deploy
=
```shell script
gcloud compute ssh instance-1
sudo systemctl stop nayok.service 
gcloud compute scp target/x86_64-unknown-linux-musl/release/nayok instance-1:~/nayok
sudo systemctl start nayok.service
```
