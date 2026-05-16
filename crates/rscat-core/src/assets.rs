use crate::settings::Runner;
use anyhow::{Context, Result};

#[derive(Clone, Debug)]
pub struct RunnerFrame {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug)]
pub struct FrameSet {
    pub runner: Runner,
    pub frames: Vec<RunnerFrame>,
}

impl FrameSet {
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn frame(&self, index: usize) -> Option<&RunnerFrame> {
        if self.frames.is_empty() {
            return None;
        }
        self.frames.get(index % self.frames.len())
    }
}

pub fn load_runner_frames(runner: Runner) -> Result<FrameSet> {
    let sources = match runner {
        Runner::Cat => CAT_FRAMES.as_slice(),
        Runner::Parrot => PARROT_FRAMES.as_slice(),
        Runner::Horse => HORSE_FRAMES.as_slice(),
    };

    let frames = sources
        .iter()
        .map(|source| decode_png(source))
        .collect::<Result<Vec<_>>>()?;

    Ok(FrameSet { runner, frames })
}

fn decode_png(bytes: &'static [u8]) -> Result<RunnerFrame> {
    let image = image::load_from_memory(bytes)
        .context("failed to decode embedded runner image")?
        .into_rgba8();
    let (width, height) = image.dimensions();

    Ok(RunnerFrame {
        rgba: image.into_raw(),
        width,
        height,
    })
}

const CAT_FRAMES: [&[u8]; 5] = [
    include_bytes!("../../../resources/runners/cat/cat_0.png"),
    include_bytes!("../../../resources/runners/cat/cat_1.png"),
    include_bytes!("../../../resources/runners/cat/cat_2.png"),
    include_bytes!("../../../resources/runners/cat/cat_3.png"),
    include_bytes!("../../../resources/runners/cat/cat_4.png"),
];

const PARROT_FRAMES: [&[u8]; 10] = [
    include_bytes!("../../../resources/runners/parrot/parrot_0.png"),
    include_bytes!("../../../resources/runners/parrot/parrot_1.png"),
    include_bytes!("../../../resources/runners/parrot/parrot_2.png"),
    include_bytes!("../../../resources/runners/parrot/parrot_3.png"),
    include_bytes!("../../../resources/runners/parrot/parrot_4.png"),
    include_bytes!("../../../resources/runners/parrot/parrot_5.png"),
    include_bytes!("../../../resources/runners/parrot/parrot_6.png"),
    include_bytes!("../../../resources/runners/parrot/parrot_7.png"),
    include_bytes!("../../../resources/runners/parrot/parrot_8.png"),
    include_bytes!("../../../resources/runners/parrot/parrot_9.png"),
];

const HORSE_FRAMES: [&[u8]; 5] = [
    include_bytes!("../../../resources/runners/horse/horse_0.png"),
    include_bytes!("../../../resources/runners/horse/horse_1.png"),
    include_bytes!("../../../resources/runners/horse/horse_2.png"),
    include_bytes!("../../../resources/runners/horse/horse_3.png"),
    include_bytes!("../../../resources/runners/horse/horse_4.png"),
];
