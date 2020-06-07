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
mod camera;
mod application;

fn main() {
    let mut system = futures::executor::block_on(system::System::new(1920, 1080));
    
    system.run();
}
