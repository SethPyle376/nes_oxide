pub struct Controller {
    pub step_mode: bool,
    pub pause: bool,
    pub quit: bool,
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            step_mode: false,
            pause: false,
            quit: false,
        }
    }
}
