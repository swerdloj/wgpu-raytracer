// See https://github.com/grenlight/idroid_demo/blob/master/src/fluid/lbm_flow.rs
// for wgpu compute shader example

macro_rules! size_of {
    // Size of type
    ($T:ty) => {
        std::mem::size_of::<$T>()
    };
    
    // (Dynamic) Size of pointed-to value
    (ref $I:ident) => {
        std::mem::size_of_val(&$I)
    };
}

mod system;
mod quad;
mod texture;
mod raytrace;

use quad::QuadBuilder;


fn main() {
    let (mut system, raytracer) = futures::executor::block_on(system::System::new(1920, 1080));

    // let test_texture = sys.create_texture_from_path("./res/aspect_ratio_rotated.png");
    // let test_texture = system.create_texture_from_path("./res/aspect_ratio.png");
    // let test_quad = system.create_quad(test_texture);

    system.run(raytracer);
}
