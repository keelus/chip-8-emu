pub trait BeepHandler {
    fn start(&mut self);
    fn stop(&mut self);
}
