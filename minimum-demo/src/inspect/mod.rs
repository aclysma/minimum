
pub trait InspectRenderDefault<T> {
    fn render(data: &[&T], label: &'static str, ui: &imgui::Ui);
    fn render_mut(data: &mut [&mut T], label: &'static str, ui: &imgui::Ui);
}

pub trait InspectRenderAsSlider<T> {
    fn render(data: &[&T], label: &'static str, ui: &imgui::Ui);
    fn render_mut(data: &mut [&mut T], label: &'static str, ui: &imgui::Ui);
}

fn get_same_or_none<'r, 'a : 'r, 'b : 'r, T : PartialEq>(data: &'a[&'b T]) -> Option<&'r T> {
    if data.len() == 0 {
        return None;
    }

    let first = data[0];
    for d in data {
        if **d != *first {
            return None;
        }
    }

    Some(data[0])
}

fn get_same_or_none_mut<T : PartialEq + Clone>(data: &mut [&mut T]) -> Option<T> {
    if data.len() == 0 {
        return None;
    }

    let first = data[0].clone();
    for d in data {
        if **d != first {
            return None;
        }
    }

    Some(first)
}

impl InspectRenderDefault<f32> for f32 {
    fn render(data: &[&f32], label: &'static str, ui: &imgui::Ui) {
        match get_same_or_none(data) {
            Some(v) => ui.text(&imgui::im_str!("{}: {}", label, data[0])),
            None => ui.text(&imgui::im_str!("{}: ", label))
        }
    }

    fn render_mut(data: &mut [&mut f32], label: &'static str, ui: &imgui::Ui) {
        let value = get_same_or_none_mut(data);

        //TODO: What to do about inconsistent values?
        let mut value = match value {
            Some(mut v) => { v },
            None => 0.0
        };

        if ui.input_float(&imgui::im_str!("{}", label), &mut value).build() {
            for d in data {
                **d = value;
            }
        }
    }
}

impl<T : InspectRenderDefault<T>> InspectRenderDefault<Option<T>> for Option<T> {
    fn render(data: &[&Option<T>], label: &'static str, ui: &imgui::Ui) {
        let d = data[0];
        match d {
            Some(value) => <T as InspectRenderDefault<T>>::render(&[value], label, ui),
            None => ui.text(&imgui::im_str!("{}: None", label)),
        };
    }

    fn render_mut(data: &mut [&mut Option<T>], label: &'static str, ui: &imgui::Ui) {
        let d = &mut data[0];
        match d {
            Some(value) => <T as InspectRenderDefault<T>>::render_mut(&mut [value], label, ui),
            None => ui.text(&imgui::im_str!("{}: None", label))
        }
    }
}

impl InspectRenderAsSlider<f32> for f32 {
    fn render(data: &[&Self], label: &'static str, ui: &imgui::Ui) {
        ui.text(&imgui::im_str!("{}: {}", label, data[0]));
    }

    fn render_mut(data: &mut [&mut Self], label: &'static str, ui: &imgui::Ui) {
        ui.slider_float(&imgui::im_str!("{}", label), data[0], -100.0, 100.0).build();
    }
}


impl InspectRenderDefault<glm::Vec2> for glm::Vec2 {
    fn render(data: &[&Self], label: &'static str, ui: &imgui::Ui) {
        ui.text(&imgui::im_str!("{}: {}", label, data[0]));
    }

    fn render_mut(data: &mut [&mut Self], label: &'static str, ui: &imgui::Ui) {
        let mut val = [data[0].x, data[0].y];
        ui.input_float2(&imgui::im_str!("{}", label), &mut val).build();
        data[0].x = val[0];
        data[0].y = val[1];
    }
}

impl InspectRenderDefault<glm::Vec3> for glm::Vec3 {
    fn render(data: &[&Self], label: &'static str, ui: &imgui::Ui) {
        ui.text(&imgui::im_str!("{}: {}", label, data[0]));
    }

    fn render_mut(data: &mut [&mut Self], label: &'static str, ui: &imgui::Ui) {
        let mut val = [data[0].x, data[0].y, data[0].z];
        ui.input_float3(&imgui::im_str!("{}", label), &mut val).build();
        data[0].x = val[0];
        data[0].y = val[1];
        data[0].z = val[2];
    }
}

#[derive(minimum_derive::Inspect)]
pub struct MyStruct {
    pub a: f32,
    pub b: f32,
    pub c: glm::Vec2,
    pub d: glm::Vec3
}

#[derive(minimum_derive::Inspect)]
pub struct MyStruct2 {

    //#[inspect(inspector = "InspectRenderAsSlider")]
    pub a: f32,
    #[inspect(inspector = "InspectRenderAsSlider", wrapping_type = "Testingf32")] //TODO: Carry these into the optionized struct
    pub b: f32,
    #[inspect(wrapping_type = "TestingVec2")]
    pub c: glm::Vec2,
    pub d: glm::Vec3,
    //#[optionize(optionized_type = MyStructOptionized)]
    pub s: MyStruct
}

struct Testingf32;
impl InspectRenderAsSlider<f32> for Testingf32 {
    fn render(data: &[&f32], label: &'static str, ui: &imgui::Ui) {
        <f32 as InspectRenderAsSlider<f32>>::render(data, label, ui);
    }
    fn render_mut(data: &mut [&mut f32], label: &'static str, ui: &imgui::Ui) {
        <f32 as InspectRenderAsSlider<f32>>::render_mut(data, label, ui);
    }
}

struct TestingVec2;
impl InspectRenderDefault<glm::Vec2> for TestingVec2 {
    fn render(data: &[&glm::Vec2], label: &'static str, ui: &imgui::Ui) {
        <glm::Vec2 as InspectRenderDefault<glm::Vec2>>::render(data, label, ui);
    }
    fn render_mut(data: &mut [&mut glm::Vec2], label: &'static str, ui: &imgui::Ui) {
        <glm::Vec2 as InspectRenderDefault<glm::Vec2>>::render_mut(data, label, ui);
    }
}
/*
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
*/

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