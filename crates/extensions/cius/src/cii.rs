//! The CII contracts of the base invoice, delegating to the walks of the core.

use crate::Invoice;
use crate::prelude::{
    Cii, Deserializable, DocumentBuilder, Error, Namespace, Parser, Serializable, Serializer, cii,
};

impl<N: Namespace + From<cii::Namespace>> Serializable<Cii, N> for Invoice {
    fn serialize(serializer: &mut Serializer<Cii, N>, builder: &mut DocumentBuilder<Self>) {
        cii::serialize(serializer, builder);
    }
}

impl<N: Namespace + From<cii::Namespace>> Deserializable<Cii, N> for Invoice {
    fn deserialize(parser: &mut Parser<Cii, N>) -> Result<DocumentBuilder<Self>, Error> {
        cii::deserialize(parser)
    }
}
