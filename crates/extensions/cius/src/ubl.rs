//! The UBL contracts of the base invoice, delegating to the walks of the core.

use crate::Invoice;
use crate::prelude::{
    Deserializable, DocumentBuilder, Error, Namespace, Parser, Serializable, Serializer, Ubl, ubl,
};

impl<N: Namespace + From<ubl::Namespace>> Serializable<Ubl, N> for Invoice {
    fn serialize(serializer: &mut Serializer<Ubl, N>, builder: &mut DocumentBuilder<Self>) {
        ubl::serialize(serializer, builder);
    }
}

impl<N: Namespace + From<ubl::Namespace>> Deserializable<Ubl, N> for Invoice {
    fn deserialize(parser: &mut Parser<Ubl, N>) -> Result<DocumentBuilder<Self>, Error> {
        ubl::deserialize(parser)
    }
}
