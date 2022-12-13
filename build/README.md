```text
docker build -t teensy4-rs/make -f build\Dockerfile .
```

```text
docker run --rm -ti -v teensy4-rs:/home/make teensy4-rs/make
```