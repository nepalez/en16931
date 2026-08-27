# Design

The library is shaped by one decision: it does not validate invoices itself. This chapter explains the choice and what follows from it.

## Why the Library Does Not Validate

Conformance of an invoice is defined by the official [Schematron] rule sets. They rely on [XPath 2.0] and [XSLT 3.0], and no mature Rust engine covers those. The proven engine, [Saxon], is a Java product. Binding it through FFI ([SaxonC]) would poison every build.

A homegrown Rust validator would be even worse. The rules change about twice a year, and only the official sets are authoritative. A reimplementation would chase them forever, and the other side would still not trust it.

So the library delegates the check to a deployed validation service. The service is battle-tested and carries the official rules.

## What Follows from That

* You deploy a validator next to the application and keep it current.
* You write the request yourself: the library hands you the XML and the `Target`.
* Your transport policy applies: authentication, retries, and timeouts stay in your code.
* A rules update is a redeployment of the service, with no code change.
* The library stays pure Rust: no `unsafe`, no FFI, no bundled stylesheets.
* Validation feedback arrives only from the service, late in the flow.

The last point is the price. The model softens it with types that reject invalid values at construction.

## The Model: Fill Everything, Filter on Output

Business facts do not depend on the data-exchange standard. Profiles restrict the standard and add no vocabulary of their own.

So the library holds one `Invoice` model, the superset of every profile. You fill the facts you have, without tracking per-profile rules. Serialization under a chosen profile drops what that profile forbids. One invoice thus serves several receivers: serialize it once per profile.

This choice bakes the set of supported standards into the core. A new standard may demand new fields of the shared model. So its support comes with a core release, not a plugin.

## Reading the Answer

A validator answers with findings addressed by [XPath]. The library resolves each address into a model-field pointer. Your interface highlights the item name of line 2, not an [XPath] string.

An address that matches nothing fails the whole check with an `Error`. A wrong extension pairing therefore surfaces loudly, never as a silent gap.

## Features

The core crate covers what every consumer needs:

* the `Invoice` model and its domain types
* both XML syntaxes, with automatic detection on parse
* more than thirty profiles
* report parsing and binding

The optional `mime` feature adds conversions for the attachment media type.

## Extensions

Validator differences live in small extension crates:

* wrappers for the answer envelopes: `en16931-kosit`, `en16931-phive`, `en16931-svrl`
* normalizers for the address spellings: `en16931-iso`, `en16931-schxslt`, `en16931-schxslt2`

Pick one wrapper for your service and one normalizer for its processor. Any pair works together. Separate crates let a new validator arrive without a core release.

The extension points are the public `Wrapper` and `Normalizer` traits. A third-party crate implements them for another service or dialect without touching the library.

Two ready services are supported: the [KoSIT validator] and the [phive]-based ones. They do not cover every standard, and the `en16931-svrl` wrapper fills that gap. It reads the bare output of any [Schematron] processor. So you can assemble a validation service of your own. The only restriction: the rule set must be known to the core crate.

## Versioning

The core follows semantic versioning, and a new profile arrives as a minor release.

Each extension crate lives on its own line, tracking its service rather than the core. Its manifest declares the compatible range of core versions. So you upgrade the core and the extensions independently, guided by that range.

The deployed validator is versioned apart from the crates. Updating it on new rule releases is the caller's duty, not the library's.

[KoSIT validator]: https://github.com/itplr-kosit/validator
[phive]: https://github.com/phax/phive
[Saxon]: https://www.saxonica.com/
[SaxonC]: https://www.saxonica.com/saxon-c/index.xml
[Schematron]: https://schematron.com/
[XPath]: https://www.w3.org/TR/xpath-31/
[XPath 2.0]: https://www.w3.org/TR/xpath20/
[XSLT 3.0]: https://www.w3.org/TR/xslt-30/
