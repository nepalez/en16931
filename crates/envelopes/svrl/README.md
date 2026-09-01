# en16931-svrl

A wrapper of the [en16931] toolkit. It reads bare [SVRL] reports with no service envelope around them. Such reports come from validation services of your own assembly.

```rust
use en16931_core::RawReport;
use en16931_schxslt::Schxslt;
use en16931_svrl::Svrl;

let report = RawReport::parse(&answer, &Svrl, &Schxslt)?;
```

Compatible with `en16931-core` 0.1. The full guide lives at the [documentation site].

[documentation site]: https://nepalez.gitbook.io/en-16931
[en16931]: https://github.com/nepalez/en16931
[SVRL]: https://schematron.com/document/3427.html
