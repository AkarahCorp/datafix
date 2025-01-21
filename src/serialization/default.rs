use super::{Codec, DefaultCodec, F64Codec};

impl DefaultCodec for f64 {
    fn codec() -> impl Codec<Self> {
        F64Codec
    }
}
