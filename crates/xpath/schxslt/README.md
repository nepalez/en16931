# en16931-schxslt

A normalizer of the [en16931] toolkit. It reads addresses in the dialect of the [SchXslt] processor, such as `Q{urn:…}Invoice[1]`.

```rust
use en16931_core::RawReport;
use en16931_schxslt::Schxslt;
use en16931_svrl::Svrl;

let report = RawReport::parse(&answer, &Svrl, &Schxslt)?;
```

Compatible with `en16931-core` 0.1. The full guide lives at the [documentation site].

[documentation site]: https://nepalez.gitbook.io/en-16931
[en16931]: https://github.com/nepalez/en16931
[SchXslt]: https://codeberg.org/schxslt/schxslt
