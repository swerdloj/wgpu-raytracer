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
pub mod quad;
mod texture;


fn main() {
    let mut sys = futures::executor::block_on(system::System::new());

    let test_texture = sys.create_texture_from_path("./res/tree.png");
    let test_quad = sys.create_quad_full_screen(test_texture);

    sys.run(test_quad);
}
