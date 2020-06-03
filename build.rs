use shaderc;

use std::path;
use std::fs;
use std::io::Write;

/*
    NOTE:

    This file is not needed. The only purpose this serves is to fix build errors caused by shaderc.
    More info here: https://github.com/google/shaderc-rs/issues/41

    However, the end user would not really need to compile shaders (they are baked into executable anyway).
    So, keeping this as-is seems like the best solution for now.
*/

// TODO: Confirm this works
const PATH_TO_YOUR_SHADERC_DIRECTORY: &'static str = "C:/Development/shaderc/lib";

fn main() {
    // Don't need to recompile the shaders for every build
    println!("cargo:rerun-if-changed=shaders/");
    
    println!("cargo:rustc-env=SHADERC_LIB_DIR={}", PATH_TO_YOUR_SHADERC_DIRECTORY);

    let mut compiler = shaderc::Compiler::new().unwrap();

    compile_shaders("./shaders", &mut compiler);
}

fn compile_and_save_shader(shader_path: &path::Path, compiler: &mut shaderc::Compiler, shader_type: shaderc::ShaderKind) {
    let source_text = fs::read_to_string(shader_path).unwrap();

    let spirv = compiler.compile_into_spirv(&source_text, shader_type, shader_path.file_name().unwrap().to_str().unwrap(), "main", None).unwrap();

    let target_path = shader_path.parent().unwrap();

    let output_path = target_path.join(format!("{}.spv", shader_path.file_name().unwrap().to_str().unwrap()));
    let mut output = fs::File::create(output_path).unwrap();
    output.write_all(spirv.as_binary_u8()).unwrap();
}

fn compile_shaders<P: AsRef<path::Path>>(path: P, compiler: &mut shaderc::Compiler) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let entry_path = entry.path();

        if entry.path().is_dir() {
            compile_shaders(entry.path(), compiler);
        } else {
            let file_extension = entry_path.extension().unwrap().to_str().unwrap();

            // Generate spirv and save
            match file_extension {
                "hlsl" => {
                    // TODO:
                    // Enforce file naming such as "shader_name.TYPE.hlsl"
                    // e.g.: "raytrace.vert.hlsl"
                    // This is the easiest way to allow both glsl and hlsl shaders
                    //
                    // Also, see: https://docs.rs/shaderc/0.6.2/shaderc/struct.CompileOptions.html#method.set_source_language
                    unimplemented!();
                }

                "spv" => {
                    // Nothing to do
                }

                "frag" => {
                    compile_and_save_shader(entry_path.as_path(), compiler, shaderc::ShaderKind::Fragment);
                }

                "vert" => {
                    compile_and_save_shader(entry_path.as_path(), compiler, shaderc::ShaderKind::Vertex);
                }

                "comp" => {
                    compile_and_save_shader(entry_path.as_path(), compiler, shaderc::ShaderKind::Compute);
                }

                _ => {
                    panic!("Unrecognized shader type at {:?}", entry_path);
                }
            }
        }
    }
}