//! Rasterization utilities.

use crate::{gpu::GpuVertex, Result, V4};
use lyon_path::{math::Point, Builder};
use lyon_tessellation::{
    geometry_builder::vertex_builder,
    FillOptions,
    FillTessellator,
    FillVertex,
    StrokeOptions,
    StrokeTessellator,
    StrokeVertex,
    VertexBuffers,
};

/// The method by which the rasterizer will rasterize the vector path.
#[derive(Debug, Clone, Copy)]
pub enum Method {
    /// In fill method, the rasterizer will treat all the area inside the path as part of the
    /// raster area. In this method, paths are automatically closed by assuming an edge from the
    /// last to the first vertex.
    Fill,
    /// In stroke method, the rasterizer will treat the area immediately adjacent the path within
    /// the given thickness as part of the rastered area. In this method, paths are left open
    /// and no edge between the last and first vertex is assumed.
    Stroke(f32),
}

pub fn raster_path(
    builder: Builder,
    method: Method,
    color: V4,
) -> Result<(Vec<GpuVertex>, Vec<u32>)> {
    match method {
        Method::Fill => {
            let mut buffers: VertexBuffers<FillVertex, u32> = VertexBuffers::new();
            let mut tessellator = FillTessellator::new();
            let result = tessellator.tessellate_path(
                &builder.build(),
                &FillOptions::default().with_tolerance(0.05),
                &mut vertex_builder(&mut buffers, |v| v),
            );
            match result {
                Ok(_) => {}
                Err(e) => panic!("Tessellation failed: {:?}", e),
            }

            Ok((
                buffers
                    .vertices
                    .into_iter()
                    .map(|v| GpuVertex {
                        vpos: [v.position.x, v.position.y],
                        vcol: [color.x, color.y, color.z, color.w],
                    })
                    .collect(),
                buffers.indices,
            ))
        }
        Method::Stroke(thickness) => {
            let mut buffers: VertexBuffers<StrokeVertex, u32> = VertexBuffers::new();
            let mut tessellator = StrokeTessellator::new();
            tessellator
                .tessellate_path(
                    &builder.build(),
                    &StrokeOptions::default()
                        .with_line_width(thickness)
                        .with_tolerance(0.05),
                    &mut vertex_builder(&mut buffers, |v| v),
                )
                .expect("TODO: wrap error");

            Ok((
                buffers
                    .vertices
                    .into_iter()
                    .map(|v| GpuVertex {
                        vpos: [v.position.x, v.position.y],
                        vcol: [color.x, color.y, color.z, color.w],
                    })
                    .collect(),
                buffers.indices,
            ))
        }
    }
}