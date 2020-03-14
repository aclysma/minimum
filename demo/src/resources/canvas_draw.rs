use skulpin::CoordinateSystemHelper;
use skulpin::skia_safe;

struct CanvasDrawContextInner {
    canvas: *mut skia_safe::Canvas,
    coordinate_system_helper: CoordinateSystemHelper,
}

#[derive(Default)]
pub struct CanvasDrawResource {
    inner: std::sync::Mutex<Option<CanvasDrawContextInner>>,
}

unsafe impl Send for CanvasDrawResource {}
unsafe impl Sync for CanvasDrawResource {}

impl CanvasDrawResource {
    pub fn begin_draw_context(
        &mut self,
        canvas: &mut skia_safe::Canvas,
        coordinate_system_helper: skulpin::CoordinateSystemHelper,
    ) {
        let mut lock = self.inner.lock().unwrap();
        *lock = Some(CanvasDrawContextInner {
            canvas: canvas as *mut skia_safe::Canvas,
            coordinate_system_helper,
        });
    }

    pub fn end_draw_context(&mut self) {
        let mut lock = self.inner.lock().unwrap();
        *lock = None;
    }

    pub fn with_canvas<F>(
        &mut self,
        f: F,
    ) where
        F: FnOnce(&mut skia_safe::Canvas, &CoordinateSystemHelper),
    {
        let lock = self.inner.lock().unwrap();
        let lock_ref = (*lock).as_ref().unwrap();
        //let x = lock_ref.as_ref().unwrap();
        let canvas = unsafe { &mut *lock_ref.canvas };
        (f)(canvas, &lock_ref.coordinate_system_helper);
    }
}
