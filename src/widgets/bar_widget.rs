use crate::color::Color;
use crate::draw::{draw_bar, draw_box, Font, ROBOTO_REGULAR};
use crate::widget::{DrawContext, DrawReport, KeyState, ModifiersState, WaitContext, Widget};

use std::sync::{Arc, Mutex};

pub trait BarWidgetImpl {
    fn wait(&mut self, ctx: &mut WaitContext);
    fn name(&self) -> &str;
    fn value(&self) -> f32;
    fn color(&self) -> Color;
    fn inc(&mut self, inc: f32);
    fn set(&mut self, val: f32);
    fn toggle(&mut self);
}

pub struct BarWidget {
    bar_impl: Box<dyn BarWidgetImpl + Send>,
    font: Font,
    font_size: u32,
    length: u32,
    dirty: Arc<Mutex<bool>>,
}

impl BarWidget {
    pub fn new_simple(
        font_size: f32,
        length: u32,
        w: Box<dyn BarWidgetImpl + Send>,
    ) -> Box<BarWidget> {
        let mut font = Font::new(&ROBOTO_REGULAR, font_size);
        font.add_str_to_cache(w.name());

        Box::new(BarWidget {
            bar_impl: w,
            dirty: Arc::new(Mutex::new(true)),
            font: font,
            font_size: font_size as u32,
            length: length,
        })
    }

    pub fn new<F>(font_size: f32, length: u32, f: F) -> Result<Box<BarWidget>, ::std::io::Error>
    where
        F: FnOnce(Arc<Mutex<bool>>) -> Result<Box<dyn BarWidgetImpl + Send>, ::std::io::Error>,
        F: Send + 'static + Clone,
    {
        let dirty = Arc::new(Mutex::new(true));
        let im = f(dirty.clone())?;

        let mut font = Font::new(&ROBOTO_REGULAR, font_size);
        font.add_str_to_cache(im.name());

        Ok(Box::new(BarWidget {
            bar_impl: im,
            dirty: dirty,
            font: font,
            font_size: font_size as u32,
            length: length,
        }))
    }
}

impl Widget for BarWidget {
    fn wait(&mut self, ctx: &mut WaitContext) {
        self.bar_impl.wait(ctx);
    }
    fn enter(&mut self) {}
    fn leave(&mut self) {}
    fn size(&self) -> (u32, u32) {
        (self.length, self.font_size)
    }
    fn draw(
        &mut self,
        ctx: &mut DrawContext,
        pos: (u32, u32),
    ) -> Result<DrawReport, ::std::io::Error> {
        let (width, height) = self.size();
        {
            let mut d = self.dirty.lock().unwrap();
            if !*d && !ctx.force {
                return Ok(DrawReport::empty(width, height));
            }
            *d = false;
        }

        let buf = &mut ctx.buf.subdimensions((pos.0, pos.1, width, height))?;
        buf.memset(ctx.bg);

        let c = Color::new(1.0, 1.0, 1.0, 1.0);
        self.font.draw_text(buf, ctx.bg, &c, self.bar_impl.name())?;

        let c = self.bar_impl.color();
        let bar_off = 5 * self.font_size;
        let mut val = self.bar_impl.value();
        draw_bar(
            &mut buf.offset((bar_off, 0))?,
            &c,
            self.length - bar_off,
            self.font_size,
            val,
        )?;
        let mut iter = 1.0;
        while val > 1.0 {
            let c = &Color::new(0.75 / iter, 0.25 / iter, 0.25 / iter, 1.0);
            val -= 1.0;
            iter += 1.0;
            draw_bar(
                &mut buf.offset((bar_off, 0))?,
                &c,
                self.length - bar_off,
                self.font_size,
                val,
            )?;
        }
        draw_box(
            &mut buf.offset((bar_off, 0))?,
            &c,
            (self.length - bar_off, self.font_size),
        )?;
        Ok(DrawReport {
            width: width,
            height: height,
            damage: vec![buf.get_signed_bounds()],
            full_damage: false,
        })
    }

    fn keyboard_input(&mut self, _: u32, _: ModifiersState, _: KeyState, _: Option<String>) {}
    fn mouse_click(&mut self, button: u32, (x, _): (u32, u32)) {
        *self.dirty.lock().unwrap() = true;
        match button {
            272 => {
                let offset = 5 * self.font_size;
                if x > offset {
                    self.bar_impl
                        .set(((x - offset) + 1) as f32 / (self.length - offset) as f32);
                }
            }
            273 => self.bar_impl.toggle(),
            x => {
                dbg!(x);
            }
        }
    }
    fn mouse_scroll(&mut self, (_, y): (f64, f64), _: (u32, u32)) {
        *self.dirty.lock().unwrap() = true;
        self.bar_impl.inc(y as f32 / -800.0);
    }
}
