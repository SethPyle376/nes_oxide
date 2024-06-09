const WIDTH: usize = 256;
const HEIGHT: usize = 240;

pub struct Frame {
    pub data: Vec<u8>,
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            data: vec![0; WIDTH * HEIGHT * 3],
        }
    }
}
