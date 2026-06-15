# Integration-test infrastructure

External validators the library is tested against (ADR-0001).

The library never validates on its own: it serializes an invoice to XML, hands it to a service, and maps the returned report back to model fields.

This directory provides the services and the official example invoices used to exercise that flow.

## Services

The `docker-compose.yml` defines two stock validators. The application builds and routes each request, while the library supplies the request parts and parses the report (ADR-0002). Here the smoke harness plays the application.

Each service returns its own report envelope, that a corresponding extension crate should unwrap (ADR-0004).

| Service | Image                                      | Report envelope           | Location dialect              |
|---------|--------------------------------------------|---------------------------|-------------------------------|
| `kosit` | `easybill/kosit-validator-xrechnung_3.0.2` | XOEV VARL `rep:report`    | prefixed `/ubl:Invoice/cac:…` |
| `phive` | `phelger/phorm`                            | phive `validationResults` | phive-specific                |

Both validators own their rules: each takes the bare invoice over its own API and applies the bundled Schematron. The library carries no stylesheets, and a bare engine that needs them in every request is not supported (ADR-0002).

## Running

```sh
cargo make env-up   # start the validators (docker compose up -d --wait)
cargo make smoke    # send the fixtures and check the verdicts
cargo make env-down # stop them and drop the volumes
```

The `smoke` task ships default service addresses (`KOSIT_URL`, `PHIVE_URL`, `PHIVE_TOKEN`), so it runs out of the box against the local validators. The fixtures reference these as `${VAR}`. To point the harness at another deployment, set those variables in the environment — they take precedence over the defaults. `docker-compose` binds its host ports from `KOSIT_PORT`/`PHIVE_PORT`, falling back to its own defaults.

## Fixtures

`fixtures/` holds one self-describing case per directory: a `body.xml` request body and an `about.json` carrying the request parameters (`url`, `headers`) and the `target` string the response must contain. The smoke test replays every case generically, so a new fixture is a new directory and never edits the script. See `fixtures/README.md` for the convention.
