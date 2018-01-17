use errors::Result;
use glium::texture::{RawImage2d, Texture2d};
use gpu::{Factory, Gpu, GpuMesh};
use image::{ImageBuffer, Rgb};
use std::rc::Rc;
use glium::draw_parameters::{DrawParameters, Smooth};
use gpu::programs::Library;
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, UniformBuffer};
use glium::Surface;
use palette::{Blend, Colora};
use poly::Point;
use tween::Tween;

pub const MAX_VORONOI_SITES: usize = 1024;

pub enum GpuShader {
    Default,
    Texture(Rc<Texture2d>),
    Voronoi(GpuVoronoi),
}

impl GpuShader {
    pub fn draw<S: Surface>(
        &self,
        lib: &Library,
        frame: usize,
        surface: &mut S,
        mesh: &GpuMesh,
    ) -> Result<()> {
        match *self {
            GpuShader::Default => Ok(surface.draw(
                mesh.vertices.as_ref(),
                mesh.indices.as_ref(),
                &lib.default_shader,
                &uniform!{
                    scale: mesh.scale,
                    root_center: mesh.root_center,
                    center: mesh.center,
                },
                &DrawParameters {
                    smooth: Some(Smooth::Nicest),
                    blend: mesh.blend,
                    ..Default::default()
                },
            )?),
            GpuShader::Texture(ref texture) => Ok(surface.draw(
                mesh.vertices.as_ref(),
                mesh.indices.as_ref(),
                &lib.texture_shader,
                &uniform!{
                    scale: mesh.scale,
                    root_center: mesh.root_center,
                    center: mesh.center,
                    matrix: [
                        [1.0, 0.0, 0.0, 0.0],
                        [0.0, 1.0, 0.0, 0.0],
                        [0.0, 0.0, 1.0, 0.0],
                        [0.0 , 0.0, 0.0, 1.0f32],
                    ],
                    tex: texture
                      .sampled()
                      .minify_filter(MinifySamplerFilter::Linear)
                      .magnify_filter(MagnifySamplerFilter::Linear)
                },
                &Default::default(),
            )?),
            GpuShader::Voronoi(ref gpu_voronoi) => {
                let mut strengths: [f32; MAX_VORONOI_SITES] = [0.0; MAX_VORONOI_SITES];
                for i in 0..(gpu_voronoi.site_count as usize) {
                    strengths[i] = gpu_voronoi.strengths[i].tween(frame);
                }
                gpu_voronoi.strengths_buffer.write(&strengths);
                Ok(surface.draw(
                    mesh.vertices.as_ref(),
                    mesh.indices.as_ref(),
                    &lib.voronoi_shader,
                    &uniform!{
                        center: mesh.center,
                        root_center: mesh.root_center,
                        scale: mesh.scale,
                        Colors: &gpu_voronoi.colors,
                        Positions: &gpu_voronoi.positions,
                        Strengths: &gpu_voronoi.strengths_buffer,
                        site_count: gpu_voronoi.site_count,
                    },
                    &DrawParameters {
                        smooth: Some(Smooth::Nicest),
                        blend: mesh.blend,
                        ..Default::default()
                    },
                )?)
            }
        }
    }
}

pub struct GpuVoronoi {
    positions: UniformBuffer<[[f32; 2]; MAX_VORONOI_SITES]>,
    strengths_buffer: UniformBuffer<[f32; MAX_VORONOI_SITES]>,
    strengths: Vec<Tween>,
    colors: UniformBuffer<[[f32; 4]; MAX_VORONOI_SITES]>,
    site_count: u32,
}

#[derive(Clone)]
pub struct VoronoiSite {
    pub strength: Tween,
    pub color: Colora,
    pub site: Point,
}

#[derive(Clone)]
pub enum Shader {
    Default,
    Texture(Rc<ImageBuffer<Rgb<u8>, Vec<u8>>>),
    Voronoi(Vec<VoronoiSite>),
}

impl Factory<Shader> for GpuShader {
    fn produce(spec: Shader, gpu: Rc<Gpu>) -> Result<Self> {
        match spec {
            Shader::Default => Ok(GpuShader::Default),
            Shader::Texture(ref spec_tex) => {
                let dims = spec_tex.dimensions();
                let img = spec_tex.as_ref().clone();
                let raw = RawImage2d::from_raw_rgb(img.into_raw(), dims);
                let tex = Rc::new(Texture2d::new(gpu.as_ref(), raw)?);
                Ok(GpuShader::Texture(tex))
            }
            Shader::Voronoi(sites) => {
                if sites.len() > MAX_VORONOI_SITES {
                    return Err(String::from("at most 1024 sites are supported").into());
                }

                let mut colors = [[0f32, 0.0, 0.0, 0.0]; MAX_VORONOI_SITES];
                let mut positions = [[0f32, 0.0]; MAX_VORONOI_SITES];
                let mut strengths = Vec::new();
                let site_count = sites.len() as u32;
                let (height, width) = gpu.display.get_framebuffer_dimensions();
                for (
                    i,
                    VoronoiSite {
                        strength,
                        color,
                        site,
                    },
                ) in sites.into_iter().enumerate()
                {
                    let cp = Colora {
                        alpha: 1.0,
                        ..color
                    }.into_premultiplied();

                    colors[i] = [cp.red, cp.green, cp.blue, color.alpha];
                    positions[i] = [site.x * (width as f32), site.y * (height as f32)];
                    strengths.push(strength)
                }

                Ok(GpuShader::Voronoi(GpuVoronoi {
                    positions: UniformBuffer::new(gpu.as_ref(), positions)?,
                    colors: UniformBuffer::new(gpu.as_ref(), colors)?,
                    strengths_buffer: UniformBuffer::new(gpu.as_ref(), [0.0; MAX_VORONOI_SITES])?,
                    strengths,
                    site_count,
                }))
            }
        }
    }
}
