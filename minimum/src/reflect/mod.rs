
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
/*
// A thing that wants to look at the type implements this
trait Inspector<T> {
    fn visit(t: &T);
}

// Deriving Reflect makes the type implement this
trait ReflectMacroTest : Sized {
    fn visit<T : Inspector<Self>>(&self);
}

trait ReflectMacro {
    //fn visit(&self);
}



//Another approach would be to have some implement inspector

// An inspector
//pub trait PrintInspector<T> {
//    fn visit(t: &T);
//}


impl Inspector<f32> for f32 {
    fn visit(t: &f32) {
        println!("it's a f32: {:?}", t);
    }
}

impl Inspector<u32> for u32 {
    fn visit(t: &u32) {
        println!("it's a u32: {:?}", t);
    }
}

//need some way to configure things


// A struct I want to visit fields of
#[derive(Reflect)]
struct MyS {
    //#[inspector(PrintInspect, prefix="asdf")]
    a: f32,
    b: u32
}

// What the macro should generate
impl ReflectMacroTest for MyS {
    fn visit<T : Inspector<Self>>(&self) {
        //Inspector::visit::<f32>(&self.a);
        //Inspector::visit::<u32>(&self.b);
        Inspector::visit::<WrapperToChangeInspectorUi>(WrapperToChangeInspectorUi(self.a))
    }
}

#[test]
fn a_test() {
    let s = MyS { a: 1.0, b: 2 };
    s.visit();
}
*/

/*
struct VisitorTypeOne {

}

trait FieldVisitor<T> {
    fn pass(t: &Self);
}

impl FieldVisitor<VisitorTypeOne> for f32 {
    fn pass(t: &Self) {

    }
}

impl<T> FieldVisitor<T> for MyStr {
    fn pass(t: &MyStr) {
        <f32 as FieldVisitor<T>>::pass(&t.a);
        <f32 as FieldVisitor<T>>::pass(&t.b);
    }
}


trait SomeTrait {
    fn pass(t: &Self);
}

impl SomeTrait for f32 {
    fn pass(t: &f32) {

    }
}

struct MyStr {
    a: f32,
    b: f32
}

impl SomeTrait for MyStr {
    fn pass(t: &MyStr) {
        <f32 as SomeTrait>::pass(&t.a);
        <f32 as SomeTrait>::pass(&t.b);
    }
}
*/