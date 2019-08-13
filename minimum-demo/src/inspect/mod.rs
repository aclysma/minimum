
use minimum::util::optionize::*;

pub trait InspectRenderDefault {
    fn render(&self, label: &'static str, ui: &imgui::Ui);
    fn render_mut(&mut self, label: &'static str, ui: &imgui::Ui);
}

pub trait InspectRenderAsSlider {
    fn render(&self, label: &'static str, ui: &imgui::Ui);
    fn render_mut(&mut self, label: &'static str, ui: &imgui::Ui);
}

impl InspectRenderDefault for f32 {
    fn render(&self, label: &'static str, ui: &imgui::Ui) {
        ui.text(&imgui::im_str!("{}: {}", label, self));
    }

    fn render_mut(&mut self, label: &'static str, ui: &imgui::Ui) {
        ui.input_float(&imgui::im_str!("{}", label), self).build();
    }
}

impl<T : InspectRenderDefault> InspectRenderDefault for Option<T> {
    fn render(&self, label: &'static str, ui: &imgui::Ui) {
        match self {
            Some(value) => InspectRenderDefault::render(value, label, ui),
            None => ui.text(&imgui::im_str!("{}: None", label)),
        };
    }

    fn render_mut(&mut self, label: &'static str, ui: &imgui::Ui) {
        match self {
            Some(value) => InspectRenderDefault::render_mut(value, label, ui),
            None => ui.text(&imgui::im_str!("{}: None", label))
        }
    }
}

impl<T : InspectRenderDefault> InspectRenderDefault for DefaultOptionized<T> {
    fn render(&self, label: &'static str, ui: &imgui::Ui) {
        InspectRenderDefault::render(&self.value, label, ui)
    }

    fn render_mut(&mut self, label: &'static str, ui: &imgui::Ui) {
        InspectRenderDefault::render_mut(&mut self.value, label, ui)
    }
}


impl InspectRenderAsSlider for f32 {
    fn render(&self, label: &'static str, ui: &imgui::Ui) {
        ui.text(&imgui::im_str!("{}: {}", label, self));
    }

    fn render_mut(&mut self, label: &'static str, ui: &imgui::Ui) {
        ui.slider_float(&imgui::im_str!("{}", label), self, -100.0, 100.0).build();
    }
}


impl InspectRenderDefault for glm::Vec2 {
    fn render(&self, label: &'static str, ui: &imgui::Ui) {
        ui.text(&imgui::im_str!("{}: {}", label, self));
    }

    fn render_mut(&mut self, label: &'static str, ui: &imgui::Ui) {
        let mut val = [self.x, self.y];
        ui.input_float2(&imgui::im_str!("{}", label), &mut val).build();
        self.x = val[0];
        self.y = val[1];
    }
}

impl InspectRenderDefault for glm::Vec3 {
    fn render(&self, label: &'static str, ui: &imgui::Ui) {
        ui.text(&imgui::im_str!("{}: {}", label, self));
    }

    fn render_mut(&mut self, label: &'static str, ui: &imgui::Ui) {
        let mut val = [self.x, self.y, self.z];
        ui.input_float3(&imgui::im_str!("{}", label), &mut val).build();
        self.x = val[0];
        self.y = val[1];
        self.z = val[2];
    }
}

#[derive(minimum_derive::Inspect, minimum_derive::Optionize)]
pub struct MyStruct {
    pub a: f32,
    pub b: f32,
    pub c: glm::Vec2,
    pub d: glm::Vec3
}

#[derive(minimum_derive::Inspect, minimum_derive::Optionize)]
pub struct MyStruct2 {

    #[inspect(inspector = "InspectRenderAsSlider")]
    pub a: f32,
    #[inspect(inspector = "InspectRenderAsSlider", wrapping_type = "Testingf32")]
    pub b: f32,
    #[inspect(wrapping_type = "TestingVec2")]
    pub c: glm::Vec2,
    pub d: glm::Vec3,
    #[optionize(optionized_type = MyStructOptionized)]
    pub s: MyStruct
}

struct Testingf32<'a>(&'a f32);
impl<'a> From<&'a f32> for Testingf32<'a> {
    fn from(value: &'a f32) -> Self {
        Self(value)
    }
}

impl<'a> InspectRenderAsSlider for Testingf32<'a> {
    fn render(&self, label: &'static str, ui: &imgui::Ui) {
        <f32 as InspectRenderAsSlider>::render(self.0, label, ui);
    }
    fn render_mut(&mut self, label: &'static str, ui: &imgui::Ui) {
        <f32 as InspectRenderAsSlider>::render(self.0, label, ui);
    }
}

struct TestingVec2<'a>(&'a glm::Vec2);
impl<'a> From<&'a glm::Vec2> for TestingVec2<'a> {
    fn from(value: &'a glm::Vec2) -> Self {
        Self(value)
    }
}

impl<'a> InspectRenderDefault for TestingVec2<'a> {
    fn render(&self, label: &'static str, ui: &imgui::Ui) {
        <glm::Vec2 as InspectRenderDefault>::render(self.0, label, ui);
    }
    fn render_mut(&mut self, label: &'static str, ui: &imgui::Ui) {
        <glm::Vec2 as InspectRenderDefault>::render(self.0, label, ui);
    }
}

struct WrappedType<'a>(&'a Option<f32>);

impl<'a> From<&'a Option<f32>> for WrappedType<'a> {
    fn from(value: &'a Option<f32>) -> Self {
        WrappedType(value)
    }
}

impl<'a> minimum::util::inspect::InspectTestTrait for WrappedType<'a> {

    fn visit(&self, label: &'static str) {
        let opt : Option<f32> = Some(1.0);
        let reffed = WrappedType::from(&opt);
    }

    fn visit_mut(&mut self, label: &'static str) {

    }
}


/*
use minimum_derive::inspect;

#[derive(Inspect)]
pub struct MyStruct2 {
    #[inspect(inspector_type = DefaultRenderer)]
    #[inspect(label = "hi")]
    #[inspect(min = 100.0, max = 200.0)]
    #[inspect(tooltip = "tessdfsd fas ")]
    pub a: f32,

    #[inspect(inspector_type = DefaultRenderer)]
    #[inspect(label = "hi")]
    #[inspect(min = 100.0, max = 200.0)]
    #[inspect(tooltip = "tessdfsd fas ")]
    pub b: f32,

    #[inspect(inspector_type = DefaultRenderer)]
    #[inspect(label = "hi")]
    #[inspect(min = 100.0, max = 200.0)]
    #[inspect(tooltip = "tessdfsd fas ")]
    pub c: glm::Vec2,

    #[inspect(inspector_type = DefaultRenderer)]
    #[inspect(label = "hi")]
    #[inspect(min = 100.0, max = 200.0)]
    #[inspect(tooltip = "tessdfsd fas ")]
    pub d: glm::Vec3
}

*/