# en16931-phive

A wrapper of the [en16931] toolkit. It reads the answers of [phive] services, such as [phorm], and names the rule set for the request URL.

```rust
use en16931_core::RawReport;
use en16931_iso::Iso;
use en16931_phive::Phive;

let rules = Phive.vendor_id(document.target())?;
// POST the XML to {base}/api/validate/{rules}:latest, then read the answer.
let report = RawReport::parse(&answer, &Phive, &Iso)?;
```

Compatible with `en16931-core` 0.1. The full guide lives at the [documentation site].

[documentation site]: https://nepalez.gitbook.io/en-16931
[en16931]: https://github.com/nepalez/en16931
[phive]: https://github.com/phax/phive
[phorm]: https://github.com/phax/phorm
