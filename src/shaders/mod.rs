pub mod rendering {
    vulkano_shaders::shader! {
        ty: "compute",
        path: r"src/shaders/rendering/render.comp"
    }
}
