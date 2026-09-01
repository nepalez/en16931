# en16931-iso

A normalizer of the [en16931] toolkit. It reads addresses in the dialect of the [ISO Schematron skeleton], such as `/ubl:Invoice/cac:InvoiceLine[2]/cbc:ID`.

```rust
use en16931_core::RawReport;
use en16931_iso::Iso;
use en16931_kosit::Kosit;

let report = RawReport::parse(&answer, &Kosit, &Iso)?;
```

Compatible with `en16931-core` 0.1. The full guide lives at the [documentation site].

[documentation site]: https://nepalez.gitbook.io/en-16931
[en16931]: https://github.com/nepalez/en16931
[ISO Schematron skeleton]: https://github.com/Schematron/schematron
