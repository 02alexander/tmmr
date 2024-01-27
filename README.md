# tmmr.nu
A website you can curl to get a simple timer.
## Usage

``` bash
$ cargo run 8000 & # binds to port 8000
$ curl localhost:8000/<hours>:<minutes>:<seconds> 
```

for example

```bash
$ curl tmmr.nu/1:50 # sets timer to 1 minute and 50 seconds.
```
