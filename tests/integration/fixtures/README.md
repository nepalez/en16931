# Fixtures

Each numbered directory is one self-describing validation case:

* `body.xml` — the request body, sent verbatim. It holds the whole body, not just the invoice.
* `about.json` — a human `title`, the `target` string the response must contain, and the request `url` and `headers`. The `url` and header values may reference environment variables as `${VAR}`.

`smoke.sh` sends `body.xml` to `url` with `headers` and checks that the response contains `target`. A new case is a new directory and never an edit of the script.

The invalid cases derive from a valid one by a single edit, recorded in the `title`. The edit breaks one calculation rule and keeps the document structurally valid. The failure therefore comes from the rules, and the `target` is the identifier of the tripped rule.
