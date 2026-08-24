use crate::prelude::*;
use Binding::*;
use DocumentKind::*;
use Profile::*;

/// Names the rule set the service checks such a document against.
pub(crate) fn try_from_target(target: Target) -> Result<&'static str, CoreError> {
    Ok(match (target.profile, target.binding, target.kind) {
        (En16931, Cii, _) => "eu.cen.en16931:cii",
        (En16931, Ubl, CreditNote) => "eu.cen.en16931:ubl-creditnote",
        (En16931, Ubl, Invoice) => "eu.cen.en16931:ubl",
        (FacturXBasic, Cii, _) => "fr.factur-x:basic",
        (FacturXExtended, Cii, _) => "fr.factur-x:extended",
        (Nlcius10, Cii, _) => "org.simplerinvoicing:nlcius-cii",
        (Nlcius10, Ubl, CreditNote) => "org.simplerinvoicing:creditnote",
        (Nlcius10, Ubl, Invoice) => "org.simplerinvoicing:invoice",
        (Nlcius10GAccount, Ubl, Invoice) => "org.simplerinvoicing:invoice20.g-account",
        (PeppolBisBilling30, Ubl, CreditNote) => "eu.peppol.bis3:creditnote",
        (PeppolBisBilling30, Ubl, Invoice) => "eu.peppol.bis3:invoice",
        (PeppolBisBillingInternationalAunz30, Ubl, CreditNote) => {
            "eu.peppol.bis3.aunz.ubl:creditnote"
        }
        (PeppolBisBillingInternationalAunz30, Ubl, Invoice) => "eu.peppol.bis3.aunz.ubl:invoice",
        (PeppolBisBillingInternationalSg30, Ubl, CreditNote) => "eu.peppol.bis3.sg.ubl:creditnote",
        (PeppolBisBillingInternationalSg30, Ubl, Invoice) => "eu.peppol.bis3.sg.ubl:invoice",
        (PeppolBisSelfBilling30, Ubl, CreditNote) => "eu.peppol.bis3:creditnote-self-billing",
        (PeppolBisSelfBilling30, Ubl, Invoice) => "eu.peppol.bis3:invoice-self-billing",
        (PeppolBisSelfBillingInternationalAunz30, Ubl, CreditNote) => {
            "eu.peppol.bis3.aunz.ubl:creditnote-self-billing"
        }
        (PeppolBisSelfBillingInternationalAunz30, Ubl, Invoice) => {
            "eu.peppol.bis3.aunz.ubl:invoice-self-billing"
        }
        (PintAeBilling, Ubl, CreditNote) => "org.peppol.pint.ae:creditnote",
        (PintAeBilling, Ubl, Invoice) => "org.peppol.pint.ae:invoice",
        (PintAeSelfBilling, Ubl, CreditNote) => "org.peppol.pint.ae:creditnote-self-billing",
        (PintAeSelfBilling, Ubl, Invoice) => "org.peppol.pint.ae:invoice-self-billing",
        (PintAunzBilling, Ubl, CreditNote) => "org.peppol.pint.aunz:creditnote",
        (PintAunzBilling, Ubl, Invoice) => "org.peppol.pint.aunz:invoice",
        (PintAunzSelfBilling, Ubl, CreditNote) => "org.peppol.pint.aunz:creditnote-self-billing",
        (PintAunzSelfBilling, Ubl, Invoice) => "org.peppol.pint.aunz:invoice-self-billing",
        (PintEuBilling, Ubl, CreditNote) => "org.peppol.pint.eu:creditnote",
        (PintEuBilling, Ubl, Invoice) => "org.peppol.pint.eu:invoice",
        (PintJpBilling, Ubl, CreditNote) => "org.peppol.pint.jp:credit-note",
        (PintJpBilling, Ubl, Invoice) => "org.peppol.pint.jp:invoice",
        (PintMyBilling, Ubl, CreditNote) => "org.peppol.pint.my:creditnote",
        (PintMyBilling, Ubl, Invoice) => "org.peppol.pint.my:invoice",
        (PintMySelfBilling, Ubl, CreditNote) => "org.peppol.pint.my:creditnote-self-billing",
        (PintMySelfBilling, Ubl, Invoice) => "org.peppol.pint.my:invoice-self-billing",
        (UblBe, Ubl, CreditNote) => "be.ubl:credit-note",
        (UblBe, Ubl, Invoice) => "be.ubl:invoice",
        (XRechnung30, Cii, _) => "de.xrechnung:cii",
        (XRechnung30, Ubl, CreditNote) => "de.xrechnung:ubl-creditnote",
        (XRechnung30, Ubl, Invoice) => "de.xrechnung:ubl-invoice",
        (XRechnung30Extension, Cii, _) => "de.xrechnung.extension:cii",
        (XRechnung30Extension, Ubl, Invoice) => "de.xrechnung.extension:ubl-invoice",
        _ => Err(CoreError::UnsupportedTarget(target))?,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    // A document of the given profile, binding, and kind.
    fn target(profile: Profile, binding: Binding, kind: DocumentKind) -> Target {
        Target {
            profile,
            binding,
            kind,
        }
    }

    #[test]
    fn names_the_rule_set_of_an_invoice() {
        let named = try_from_target(target(PeppolBisBilling30, Ubl, Invoice));

        assert_eq!(named.ok(), Some("eu.peppol.bis3:invoice"));
    }

    #[test]
    fn names_the_rule_set_of_a_credit_note() {
        let named = try_from_target(target(PeppolBisBilling30, Ubl, CreditNote));

        assert_eq!(named.ok(), Some("eu.peppol.bis3:creditnote"));
    }

    #[test]
    fn names_a_rule_set_per_binding() {
        let ubl = try_from_target(target(XRechnung30, Ubl, Invoice));
        let cii = try_from_target(target(XRechnung30, Cii, Invoice));

        assert_eq!(ubl.ok(), Some("de.xrechnung:ubl-invoice"));
        assert_eq!(cii.ok(), Some("de.xrechnung:cii"));
    }

    #[test]
    fn names_one_rule_set_for_both_kinds_of_a_cii_document() {
        let invoice = try_from_target(target(En16931, Cii, Invoice)).ok();
        let credit_note = try_from_target(target(En16931, Cii, CreditNote)).ok();

        assert_eq!(invoice, credit_note);
        assert_eq!(invoice, Some("eu.cen.en16931:cii"));
    }

    #[test]
    fn rejects_a_profile_the_service_does_not_carry() {
        let named = try_from_target(target(CiusIt200, Ubl, Invoice));

        assert!(matches!(named, Err(CoreError::UnsupportedTarget(_))));
    }

    #[test]
    fn rejects_a_binding_the_profile_does_not_reach() {
        let named = try_from_target(target(PeppolBisBilling30, Cii, Invoice));

        assert!(matches!(named, Err(CoreError::UnsupportedTarget(_))));
    }

    #[test]
    fn rejects_a_credit_note_the_service_does_not_carry() {
        let named = try_from_target(target(XRechnung30Extension, Ubl, CreditNote));

        assert!(matches!(named, Err(CoreError::UnsupportedTarget(_))));
    }
}
