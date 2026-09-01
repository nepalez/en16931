# Building and Deploying Validators

This chapter explains how to start the ready validators and how to build a service of your own.

## The Ready Services

The repository ships a ready `tests/integration/docker-compose.yml`. `cargo make env-up` starts both of its services and waits for their readiness:

* `kosit` answers on host port `8082` (rebind with `KOSIT_PORT`),
* `phive` answers on host port `8083` (rebind with `PHIVE_PORT`) and requires the `PHIVE_TOKEN` in every request.

The images and their versions are pinned in the file. `cargo make env-down` stops the services and drops their volumes.

`cargo make smoke` sends the official example invoices and checks the results. Override `KOSIT_URL`, `PHIVE_URL`, and `PHIVE_TOKEN` to point the same checks at a deployment of your own.

## Building a Service of Your Own

For a standard the ready services do not carry, build a service from a [Schematron] engine and the rules of your profile.

### Installation of the Necessary Tools

Install the parts first:

* a Java runtime, such as [Temurin] — runs the two jars below,
* the [Saxon-HE] jar from its releases — executes a precompiled stylesheet against an invoice,
* the [SchXslt] command line jar from its releases — checks an invoice against bare `.sch` sources, needed only when step 1 below gives no precompiled stylesheet,
* [Docker] — packages the service into a container.

### Building the Service

1. **Take the rules.**

    The publisher of your profile releases its official [Schematron] rules. The releases of the [EN-16931 rules], for example, ship both the `.sch` sources and precompiled [XSLT] stylesheets for [UBL] and [CII]. Prefer a precompiled stylesheet.

2. **Produce a report locally.** 

    Run the check on an example invoice. With a precompiled stylesheet, [Saxon-HE] transforms the invoice into the report:

    ```sh
    java -cp saxon-he.jar net.sf.saxon.Transform -s:invoice.xml -xsl:rules.xslt -o:report.svrl
    ```

    With bare `.sch` sources, [SchXslt] validates in one call:

    ```sh
    java -jar schxslt-cli.jar -d invoice.xml -s rules.sch -o report.svrl
    ```

3. **Wrap the run in HTTP.** 
 
    Write a small endpoint in any stack. It saves the body of a request to a file, runs the command of step 2 on that file, and returns the produced report as the body of the response.

4. **Package the container.**

    Start the `Dockerfile` from a [Temurin] image, copy the jars, the stylesheet, and the endpoint of step 3, and expose the port of the endpoint.

    Deploy the container next to the application, as with the ready services.

5. **Wire the application.**

    The service answers with a bare [SVRL] report, so the application reads it with `en16931-svrl`. The normalizer follows the tool of step 2 — `en16931-iso` for the precompiled stylesheets, `en16931-schxslt` for a [SchXslt] run.

[CII]: https://en.wikipedia.org/wiki/UN/CEFACT
[Docker]: https://docs.docker.com/
[EN-16931 rules]: https://github.com/ConnectingEurope/eInvoicing-EN16931
[Saxon-HE]: https://github.com/Saxonica/Saxon-HE
[SchXslt]: https://codeberg.org/schxslt/schxslt
[Schematron]: https://schematron.com/
[SVRL]: https://schematron.com/document/3427.html
[Temurin]: https://adoptium.net/
[UBL]: https://www.oasis-open.org/standard/ublv2-1/
[XSLT]: https://www.w3.org/TR/xslt-30/
