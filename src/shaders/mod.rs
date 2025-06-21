
pub mod terrain_gen {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/terrain_gen/terrain_generator.comp",
        linalg_type: "nalgebra",
    }
}
pub mod rendering {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/rendering/render.comp",
        linalg_type: "nalgebra",
    }
}