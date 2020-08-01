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
export TARGET_CC="x86_64-unknown-linux-gnu-gcc"
export TARGET_CFLAGS="-I $(pwd)/usr/include/x86_64-linux-gnu -isystem $(pwd)/usr/include"
export LD_LIBRARY_PATH="$(pwd)/usr/lib/x86_64-linux-gnu;$(pwd)/lib/x86_64-linux-gnu"
export OPENSSL_DIR="$(pwd)/usr/"
export OPENSSL_LIB_DIR="$(pwd)/usr/lib/x86_64-linux-gnu/"

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
