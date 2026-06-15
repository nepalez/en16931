# Fixtures

Each numbered directory is one self-describing validation case:

* `body.xml` — the request body, sent verbatim. It holds the whole body, not just the invoice, so a service that wraps the document keeps its wrapper here.
* `about.json` — the case: a human `title`, the `target` string the response must contain, and the request parameters apart from the body (`url` and a `headers` array). The `url` and header values may reference environment variables as `${VAR}`, which the caller provides (the `smoke` task ships defaults).

`smoke.sh` replays every case: it sends `body.xml` to `url` with `headers` and checks that the response contains `target`. Adding a case is just a new directory, and a new service is just different request parameters, so neither ever touches the script.

The invalid cases are derived from a valid one by a single documented edit (recorded in the `title`) that breaks one calculation rule. They stay XSD-valid, so the failure comes from Schematron, not the schema, and the `target` is the tripped business-rule id.
