# en16931-core

The core of the [en16931] toolkit. The crate holds the typed `Invoice` model. It serializes the model into [UBL] or [CII] under a chosen profile, parses incoming documents, and binds validator findings to model fields.

```rust
use en16931_core::{Binding, Document, DocumentBuilder, Profile};

let document = Document::try_from(DocumentBuilder {
    invoice,
    profile: Profile::XRechnung30,
    binding: Binding::Ubl,
    business_process: None,
})?;
let xml = document.xml();
```

The crate follows semantic versioning. The full guide lives at the [documentation site].

[CII]: https://en.wikipedia.org/wiki/UN/CEFACT
[documentation site]: https://nepalez.gitbook.io/en-16931
[en16931]: https://github.com/nepalez/en16931
[UBL]: https://www.oasis-open.org/standard/ublv2-1/
