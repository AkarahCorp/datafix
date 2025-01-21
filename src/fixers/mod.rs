use crate::dynamic::Dynamic;

pub trait DataFixerRule {
    fn fix_dyn(&self, value: &mut Dynamic);
}
