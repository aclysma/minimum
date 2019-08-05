use std::marker::PhantomData;

// This walks a visitor through values
trait Reflector<T> {
    fn type_name() -> &'static str;

    fn reflect_visitor(instance: &T, visitor: &dyn ReflectVisitor);
    fn reflect_visitor_mut(instance: &mut T, visitor: &dyn ReflectVisitorMut);
}

trait ReflectVisitor {
    fn visit_string(&self, name: &'static str, value: &String);
}


trait ReflectVisitorMut {
    fn visit_string(&self, name: &'static str, value: &mut String);
}

mod test {
    use super::*;

    struct TestStruct {
        str1: String,
        str2: String,
    }

    impl TestStruct {
        fn new(str1: String, str2: String) -> Self {
            TestStruct {
                str1,
                str2
            }
        }
    }

    struct TestStructReflector;
    impl Reflector<TestStruct> for TestStructReflector {

        fn type_name() -> &'static str {
            "TestStruct"
        }

        fn reflect_visitor(instance: &TestStruct, visitor: &ReflectVisitor) {
            visitor.visit_string("str1", &instance.str1);
            visitor.visit_string("str2", &instance.str2);
        }

        fn reflect_visitor_mut(instance: &mut TestStruct, visitor: &ReflectVisitorMut) {
            visitor.visit_string("str1", &mut instance.str1);
            visitor.visit_string("str2", &mut instance.str2);
        }
    }

    #[test]
    fn test_thing() {
        let mut test_struct1 = TestStruct::new("abc".to_string(), "edf".to_string());
        let mut test_struct2 = TestStruct::new("123".to_string(), "456".to_string());

        struct Visitor;
        impl ReflectVisitorMut for Visitor {
            fn visit_string(&self, name: &'static str, value: &mut String) {

            }
        }

        TestStructReflector::reflect_visitor_mut(&mut test_struct1, &Visitor);
        TestStructReflector::reflect_visitor_mut(&mut test_struct2, &Visitor);
    }
}






