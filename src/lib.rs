// Public
pub mod canvas;
pub mod point;

pub trait AsPainter {
    fn redraw(&mut self);

    fn push_point(&mut self, pt: point::Point);

    fn start_line(&mut self, pt: point::Point);

    fn end_line(&mut self);

    fn cancle_line(&mut self);

    fn set_aspect(&mut self, aspect: f32);
}
