

pub trait OptionizedMember<T> : Sized {
    fn empty() -> Self;

    fn assign_to_optionized(dest: &mut Self, source: &T);
    fn merge_to_optionized(dest: &mut Self, source: &T);

    fn new(source: &T) -> Self {
        let mut v = Self::empty();
        Self::assign_to_optionized(&mut v, source);
        v
    }
}

pub struct DefaultOptionized<T> {
    pub value: Option<T>
}


impl<T : Copy + PartialEq> OptionizedMember<T> for DefaultOptionized<T> {
    fn empty() -> Self {
        Self {
            value: None
        }
    }

    fn assign_to_optionized(dest: &mut DefaultOptionized<T>, source: &T) {
        dest.value = Some(*source);
    }

    fn merge_to_optionized(dest: &mut DefaultOptionized<T>, source: &T) {
        if let Some(d) = dest.value.as_ref() {
            if *d != *source {
                dest.value = None
            }
        }
    }
}

pub fn create_optionized_from_set<OptionizedT, T>(source: &[&T]) -> OptionizedT
    where
        OptionizedT : OptionizedMember<T>
{
    let mut optionized = OptionizedT::new(&source[0]);
    //OptionizedMember::to_optionized(&mut optionized, &source[0]);

    for i in 1..source.len() {
        OptionizedMember::merge_to_optionized(&mut optionized, &source[i]);
    }

    optionized
}
