

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



/*
pub trait InspectRenderRanged {
    fn render(&self, label: &'static str, ui: &imgui::Ui);
    fn render_mut(&mut self, label: &'static str, ui: &imgui::Ui);
}

impl InspectRenderRanged for f32 {
    fn render(&self, label: &'static str, ui: &imgui::Ui) {
        ui.text(&imgui::im_str!("{}: {}", label, self));
    }

    fn render_mut(&mut self, label: &'static str, ui: &imgui::Ui) {
        ui.input_float(&imgui::im_str!("{}", label), self).build();
    }
}
*/

#[derive(minimum_derive::Inspect)]
pub struct MyStruct {
    pub a: f32,
    pub b: f32,
    pub c: glm::Vec2,
    pub d: glm::Vec3
}
/*
impl InspectRenderDefault for MyStruct {
    fn render(&self, _label: &'static str, ui: &imgui::Ui) {
        let header = ui.collapsing_header(imgui::im_str!("MyStruct")).build();
        ui.indent();
        InspectRenderDefault::render(&self.a, "a", ui);
        InspectRenderAsSlider::render(&self.b, "b", ui);
        InspectRenderDefault::render(&self.c, "c", ui);
        InspectRenderDefault::render(&self.d, "d", ui);
        ui.unindent();
    }

    fn render_mut(&mut self, _label: &'static str, ui: &imgui::Ui) {
        let header = ui.collapsing_header(imgui::im_str!("MyStruct")).build();
        ui.indent();
        InspectRenderDefault::render_mut(&mut self.a, "a", ui);
        InspectRenderAsSlider::render_mut(&mut self.b, "b", ui);
        InspectRenderDefault::render_mut(&mut self.c, "c", ui);
        InspectRenderDefault::render_mut(&mut self.d, "d", ui);
        ui.unindent();
    }
}
*/

#[derive(minimum_derive::Inspect)]
pub struct MyStruct2 {

    #[inspect(inspector = InspectRenderAsSlider)]
    pub a: f32,
    pub b: f32,
    pub c: glm::Vec2,
    pub d: glm::Vec3,
    pub s: MyStruct
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