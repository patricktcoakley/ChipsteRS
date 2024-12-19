#[derive(Debug, PartialEq, Copy, Clone)]
pub enum State {
    Running,
    Paused,
    Finished,
    Off,
}
