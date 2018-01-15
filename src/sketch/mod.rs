use composition::Composition;
use errors::Result;
use glium::glutin::WindowEvent;
use gpu::{Factory, Gpu, Render};
use rand::{random, SeedableRng, StdRng};
use std::{fs, thread, time};
use std::rc::Rc;

pub struct SketchCfg {
    pub size: u32,
    pub root_frame_filename: Option<String>,
    pub frame_limit: usize,
    pub seed: Option<usize>,
    pub still: bool,
}

impl Default for SketchCfg {
    fn default() -> Self {
        Self {
            size: 400,
            root_frame_filename: None,
            frame_limit: 400,
            seed: None,
            still: false,
        }
    }
}

pub struct SketchContext {
    pub cfg: SketchCfg,
    pub gpu: Rc<Gpu>,
    pub frame: usize,
    pub current_seed: usize,
}

impl SketchContext {
    pub fn produce<Spec, F: Factory<Spec>>(&self, spec: Spec) -> Result<F> {
        F::produce(spec, self.gpu.clone())
    }
}

pub trait Sketch {
    fn sketch(&self, ctx: &SketchCfg, rng: StdRng) -> Result<Composition>;
}

pub fn sketch<S: Sketch>(cfg: SketchCfg, sketch: S) -> Result<()> {
    let (gpu, events_loop) = Gpu::new(cfg.size)?;
    let current_seed = cfg.seed.unwrap_or(random());
    let mut context = SketchContext {
        cfg,
        gpu: Rc::new(gpu),
        frame: 0,
        current_seed,
    };
    let mut render = Render::produce(
        sketch.sketch(&context.cfg, StdRng::from_seed(&[context.current_seed]))?,
        context.gpu.clone(),
    )?;

    let mut cycle = Gpu::events(events_loop);
    while let Some((events_loop, events)) = cycle {
        if events
            .iter()
            .find(|event| match **event {
                WindowEvent::ReceivedCharacter('r') => true,
                _ => false,
            })
            .is_some()
        {
            context.current_seed = random();
            context.frame = 0;
            render = Render::produce(
                sketch.sketch(&context.cfg, StdRng::from_seed(&[context.current_seed]))?,
                context.gpu.clone(),
            )?;
        }
        if !(context.cfg.still && context.frame > 0) {
            render = render.step(context.frame)?;
            context.gpu.draw(context.frame, render.render());
            if let Some(ref root_frame_filename) = context.cfg.root_frame_filename {
                if context.frame < context.cfg.frame_limit {
                    let saves_dir = format!("{}/{:14}/", root_frame_filename, context.current_seed);
                    fs::create_dir_all(&saves_dir)?;
                    context
                        .gpu
                        .save_frame(&format!("{}{:08}", saves_dir, context.frame))?;
                }
            }
        }
        cycle = Gpu::events(events_loop);
        thread::sleep(time::Duration::from_millis(32));
        context.frame += 1;
    }
    Ok(())
}
