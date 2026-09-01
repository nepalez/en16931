# Integration-test infrastructure

External validators the library is tested against.

## Layout

* `docker-compose.yml` — the `kosit` and `phive` validator services.
* `smoke.sh` — replays the cases of `fixtures/` against the running services.
* `fixtures/` — one directory per validation case, see `fixtures/README.md`.

## Running

```sh
cargo make env-up   # start the validators
cargo make smoke    # send the fixtures and check the verdicts
cargo make env-down # stop them and drop the volumes
```

The `smoke` task ships default values for `KOSIT_URL`, `PHIVE_URL`, and `PHIVE_TOKEN`. Override them to check another deployment. The compose file binds the host ports from `KOSIT_PORT` and `PHIVE_PORT`.
