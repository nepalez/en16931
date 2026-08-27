# Electronic Invoicing in Europe

This chapter introduces the world the library operates in: the standard, its XML syntaxes, the profiles, and the validation ecosystem.

## The Standard

Electronic invoicing became mandatory in EU public procurement with [Directive 2014/55/EU]. The directive obliges every contracting authority to receive and process such invoices.

The standard [EN-16931], published by the European Committee for Standardization ([CEN]), defines what an invoice states.

Every statement has a stable code:

* `BT-n`, a business term, is a single field, such as the invoice number (`BT-1`)
* `BG-n`, a business group, is a group of related terms, such as the seller (`BG-4`)

The chapters of this guide, and the library itself, refer to these codes throughout.

## Two Syntaxes, One Content

An invoice travels as an XML document. The standard admits exactly two XML syntaxes for the same content:

* [UBL] — the Universal Business Language, maintained by the standards consortium [OASIS]
* [CII] — the Cross Industry Invoice, maintained by [UN/CEFACT], the United Nations body for trade facilitation and electronic business

The directive keeps the list of syntaxes closed, and each syntax ships an XSD schema for structural validation.

The same facts take different shapes in the two syntaxes.

<details>

<summary>The invoice number (BT-1) and the issue date (BT-2) in UBL</summary>

```xml
<Invoice xmlns="urn:oasis:names:specification:ubl:schema:xsd:Invoice-2"
         xmlns:cbc="urn:oasis:names:specification:ubl:schema:xsd:CommonBasicComponents-2">
  <cbc:ID>INV-2026-001</cbc:ID>
  <cbc:IssueDate>2026-01-15</cbc:IssueDate>
</Invoice>
```

</details>

<details>

<summary>The same two terms in CII</summary>

```xml
<rsm:CrossIndustryInvoice xmlns:rsm="urn:un:unece:uncefact:data:standard:CrossIndustryInvoice:100"
                          xmlns:ram="urn:un:unece:uncefact:data:standard:ReusableAggregateBusinessInformationEntity:100"
                          xmlns:udt="urn:un:unece:uncefact:data:standard:UnqualifiedDataType:100">
  <rsm:ExchangedDocument>
    <ram:ID>INV-2026-001</ram:ID>
    <ram:IssueDateTime>
      <udt:DateTimeString format="102">20260115</udt:DateTimeString>
    </ram:IssueDateTime>
  </rsm:ExchangedDocument>
</rsm:CrossIndustryInvoice>
```

</details>

The element names, the namespaces, the nesting, and even the date format differ.

The library calls a syntax a **binding** and supports both.

## Profiles

Countries and trade networks tailor the base standard to their needs. The standard calls such a specification a [CIUS], a Core Invoice Usage Specification.

A [CIUS] restricts the standard: it forbids some optional terms, requires others, and tightens code lists. An extension goes the other way and adds information the core does not carry.

Well-known profiles include:

* [Peppol BIS Billing], which serves the international [Peppol] delivery network
* [XRechnung], which adapts the standard to German public procurement
* [NLCIUS], which narrows it down for the Netherlands
* [Factur-X], which pairs a [CII] document with a human-readable PDF for France and Germany
* [PINT], the [Peppol] International model, which localizes billing beyond Europe

Further profiles cover Austria, Belgium, Denmark, Italy, Spain, and other countries and sectors. The library supports more than thirty of them.

A document declares its profile in `BT-24`, the specification identifier (URN), such as `urn:cen.eu:en16931:2017`. The receiver reads it to learn which rules the document claims to satisfy.

## Schematron Rules

Regulators publish their rules in [Schematron] (a rule language for XML) in the form of numbered [XPath] assertions, such as `BR-CO-16`. Typical examples are:

* the arithmetic of totals
* the membership of a code in a list
* a dependency between distant fields

The rules change about twice a year, plus minor versions. Each rule release names an effective date, shortly after publication. From that date, the exchange must run on the new version of the rules.

<details>

<summary>A slightly simplified UBL rule for the amount due (BR-CO-16)</summary>

```xml
<rule context="cac:LegalMonetaryTotal">
  <assert id="BR-CO-16" flag="fatal"
          test="cbc:PayableAmount = cbc:TaxInclusiveAmount - cbc:PrepaidAmount + cbc:PayableRoundingAmount">
    [BR-CO-16] Amount due for payment (BT-115) = Invoice total amount with VAT (BT-112)
    - Paid amount (BT-113) + Rounding amount (BT-114).
  </assert>
</rule>
```

</details>

## From Rules to Stylesheets: The Role of Compilers

[Schematron] rules are not executable on their own. A compiler translates a rule set into an [XSLT] stylesheet — the executable form that a **validation engine** can run.

Three [Schematron] compilers dominate the field. They differ in how their stylesheets spell the [XPath] addresses of checked nodes:

* the classic [ISO Schematron skeleton] writes prefixed steps, such as `/ubl:Invoice/cac:InvoiceLine[2]/cbc:ID`
* [SchXslt] spells every namespace out, as in `/Q{urn:…}Invoice[1]/Q{urn:…}InvoiceLine[2]/Q{urn:…}ID[1]`
* [SchXslt 2] keeps that shape with small quirks, such as a bare attribute step `@schemeID`

<details>

<summary>The BR-CO-16 rule as represented by the ISO Schematron skeleton</summary>

```xml
<xsl:template match="cac:LegalMonetaryTotal" mode="M15">
  <xsl:choose>
    <xsl:when test="cbc:PayableAmount = cbc:TaxInclusiveAmount - cbc:PrepaidAmount + cbc:PayableRoundingAmount"/>
    <xsl:otherwise>
      <svrl:failed-assert id="BR-CO-16" flag="fatal">
        <xsl:attribute name="location">
          <xsl:apply-templates select="." mode="schematron-get-full-path"/>
        </xsl:attribute>
        <svrl:text>[BR-CO-16] Amount due for payment (BT-115) = …</svrl:text>
      </svrl:failed-assert>
    </xsl:otherwise>
  </xsl:choose>
</xsl:template>
```

</details>

## Validation

A validation engine takes two inputs, the invoice and the compiled stylesheet, and produces a report of rule violations.

The most mature engine is [Saxon], written in Java. No other ecosystem has sustained the upkeep that evolving rule sets demand. The rules lean on modern [XPath] features, so the engine must keep pace.

The report comes in [SVRL], a validation report language. Every finding in the report carries the fired rule, a message, and an [XPath] address of the offending document node.

<details>

<summary>A breach of BR-CO-16 in the report</summary>

```xml
<svrl:failed-assert id="BR-CO-16" flag="fatal"
                    test="cbc:PayableAmount = cbc:TaxInclusiveAmount - cbc:PrepaidAmount + cbc:PayableRoundingAmount"
                    location="/ubl:Invoice/cac:LegalMonetaryTotal[1]">
  <svrl:text>[BR-CO-16] Amount due for payment (BT-115) = …</svrl:text>
</svrl:failed-assert>
```

</details>

## Validation Services

Any tool that checks an invoice is loosely called a validator. A bare engine is the simplest one: it holds no rule sets of its own, but executes whatever one it receives, and its output is the plain [SVRL] report. Such a validator is inconvenient: the caller must pick the right rule set for the profile and the syntax, compile it, and refresh it on every release.

Ready-to-run applications solve this: they bundle an engine with the official rule sets, properly versioned, and expose the check as a service:

* the [KoSIT validator], the [XRechnung] reference tool from the German coordination office for IT standards ([KoSIT])
* services built on [phive], an open-source validation engine carrying the [Peppol] and [EN-16931] rule sets

Every service answers in a format of its own, wrapping the findings in an envelope. The [KoSIT validator] writes an [XOEV VARL] report, while [phive] services answer in the format of [phorm].

<details>

<summary>The same BR-CO-16 breach in the XOEV VARL report</summary>

```xml
<rep:report xmlns:rep="http://www.xoev.de/de/validator/varl/1" valid="false">
  <rep:scenarioMatched>
    <rep:validationStepResult id="val-sch.1" valid="false">
      <rep:message id="val-sch.1.1" level="error" code="BR-CO-16"
                   xpathLocation="/ubl:Invoice/cac:LegalMonetaryTotal[1]">[BR-CO-16] Amount due for payment (BT-115) = …</rep:message>
    </rep:validationStepResult>
  </rep:scenarioMatched>
</rep:report>
```

</details>

<details>

<summary>The same finding in the phorm format</summary>

```xml
<validationResults>
  <success>false</success>
  <result>
    <artifactType>schematron-xslt</artifactType>
    <item>
      <errorLevel>ERROR</errorLevel>
      <errorID>BR-CO-16</errorID>
      <errorFieldName>/:Invoice[1]/cac:LegalMonetaryTotal[1]</errorFieldName>
      <errorText>[BR-CO-16] Amount due for payment (BT-115) = …</errorText>
    </item>
  </result>
</validationResults>
```

</details>

## Exchanging an Invoice

Both sides of an exchange check the same document.

The seller fills the model with business facts and serializes it under the profile the buyer expects. A validator checks the XML before it goes to the other party.

The buyer receives the XML and checks it at a validator of their own. The valid document can be read along with the validator's warnings, if any.

A cross-border sale may involve two profiles at once. A Dutch seller works under [NLCIUS], while the buyer requires [Peppol BIS Billing]. One invoice holds the shared content. The seller serializes it twice, once per profile, and validates each document separately.

A buyer may also re-invoice the received goods to a third party. The received document then turns back into a business object. The former buyer becomes the seller of a new invoice. The new document goes out under the profile of its own receiver and is subject to a new set of rules.

## The Library's Place

The library reimplements none of this machinery. It stands between the application and a validator:

* it holds a typed invoice model and serializes it under a chosen profile and binding;
* it parses an incoming XML document back into the model;
* it reads the report of a validator and binds every finding to a model field.

Both the transport and the orchestration are up to the application using the library.

[CEN]: https://en.wikipedia.org/wiki/European_Committee_for_Standardization
[CII]: https://en.wikipedia.org/wiki/UN/CEFACT
[CIUS]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108937/CIUS+and+Extension+-+What+is+allowed
[Directive 2014/55/EU]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108867/European+legislation+on+eInvoicing
[EN-16931]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108950/EN+16931+compliance
[Factur-X]: https://fnfe-mpe.org/factur-x/factur-x_en/
[ISO Schematron skeleton]: https://github.com/Schematron/schematron
[KoSIT]: https://www.xoev.de/
[KoSIT validator]: https://github.com/itplr-kosit/validator
[NLCIUS]: https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108895/eInvoicing+in+The+Netherlands
[OASIS]: https://www.oasis-open.org/
[Peppol]: https://peppol.org/
[Peppol BIS Billing]: https://docs.peppol.eu/poacc/billing/3.0/
[phive]: https://github.com/phax/phive
[phorm]: https://github.com/phax/phorm
[PINT]: https://docs.peppol.eu/poac/pint/pint/bis/
[Saxon]: https://www.saxonica.com/
[SchXslt]: https://codeberg.org/schxslt/schxslt
[SchXslt 2]: https://codeberg.org/schxslt/schxslt2
[Schematron]: https://schematron.com/
[SVRL]: https://schematron.com/document/3427.html
[UBL]: https://www.oasis-open.org/standard/ublv2-1/
[UN/CEFACT]: https://en.wikipedia.org/wiki/UN/CEFACT
[XOEV VARL]: https://github.com/itplr-kosit/validator-configuration-xrechnung/blob/master/src/default-report.xsl
[XPath]: https://www.w3.org/TR/xpath-31/
[XRechnung]: https://xeinkauf.de/xrechnung/
[XSLT]: https://www.w3.org/TR/xslt-30/
