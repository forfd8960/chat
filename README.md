# Rust Axum -> Chat

## Gen key

```sh
openssl genpkey -algorithm ed25519 -out private.pem
```

## Gen public key

```sh
openssl pkey -in private.pem -pubout -out public.pem
```