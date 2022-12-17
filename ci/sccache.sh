#!/usr/env/bin bash

curl --output sccache.tar.gz --location https://github.com/mozilla/sccache/releases/download/v0.3.1/sccache-v0.3.1-x86_64-unknown-linux-musl.tar.gz; 
tar -xvf sccache.tar.gz; 
cp sccache-v0.3.1-x86_64-unknown-linux-musl/sccache /usr/local/bin/sccache;
chmod u+x /usr/local/bin/sccache
