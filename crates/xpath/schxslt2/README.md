# en16931-schxslt2

A normalizer of the [en16931] toolkit. It reads addresses in the dialect of the [SchXslt 2] processor, with quirks such as a bare attribute `@schemeID`.

```rust
use en16931_core::RawReport;
use en16931_schxslt2::Schxslt2;
use en16931_svrl::Svrl;

let report = RawReport::parse(&answer, &Svrl, &Schxslt2)?;
```

Compatible with `en16931-core` 0.1. The full guide lives at the [documentation site].

[documentation site]: https://nepalez.gitbook.io/en-16931
[en16931]: https://github.com/nepalez/en16931
[SchXslt 2]: https://codeberg.org/schxslt/schxslt2
