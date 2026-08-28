# Use Cases

This chapter shows the library at work in two exchange scenarios.

Each scenario is covered by a complete, compilable example in the `examples/` directory of the repository. Both examples talk to a live validator: start the services first, then run the printed command.

## Issuing an Invoice

The seller fills the `Invoice` model with business facts. The `DocumentBuilder` stages it for one profile and one binding, here [XRechnung] over UBL, and `Document::try_from` renders the XML.

The application owns the transport. The example posts the XML to the [KoSIT validator] with a plain HTTP client. The answer comes back through a pair of extensions: the `Kosit` wrapper opens the envelope of the service, and the `Iso` normalizer reads the addresses its processor writes.

`Document::check` binds every finding to a field of the model and splits the outcome. A `ValidDocument` may still carry warnings and remarks. An `InvalidDocument` carries at least one error, and each problem names the model field it points at.

The full code lives in [`examples/send_invoice.rs`]:

```sh
cargo run -p en16931-examples --example send_invoice
```

## Receiving an Invoice

Receiving works in the opposite direction: the input is the XML text that arrived from the seller, embedded in the example as the `RECEIVED_XML` constant. `Document::parse` reads the text and detects the binding automatically from the root element, so the receiver needs no prior knowledge of the syntax (here the XML happens to be CII).

The receiver checks the document at a validator of their own, here the [phive] service. Every problem of the report carries a `Context`, a dot-separated path to the model field, such as `lines[2].item.name`. An interface highlights that field instead of printing an XPath string. On success, `Invoice::from` takes the business object out of the document.

The full code lives in [`examples/receive_invoice.rs`]:

```sh
cargo run -p en16931-examples --example receive_invoice
```

[`examples/receive_invoice.rs`]: https://github.com/nepalez/en16931/blob/master/examples/receive_invoice.rs
[`examples/send_invoice.rs`]: https://github.com/nepalez/en16931/blob/master/examples/send_invoice.rs
[KoSIT validator]: https://github.com/itplr-kosit/validator
[phive]: https://github.com/phax/phive
[XRechnung]: https://xeinkauf.de/xrechnung/
