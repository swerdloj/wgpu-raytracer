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

// TODO: This doesn't seem to work. Windows path variable is still required.
// const PATH_TO_YOUR_SHADERC_DIRECTORY: &'static str = "C:/Development/shaderc/lib";

fn main() {
    // Don't need to recompile the shaders for every build
    // println!("cargo:rerun-if-changed=shaders/");
    
    // println!("cargo:rustc-env=SHADERC_LIB_DIR={}", PATH_TO_YOUR_SHADERC_DIRECTORY);

    let mut compiler = shaderc::Compiler::new().unwrap();

    compile_shaders("./shaders", &mut compiler);
}

fn compile_and_save_shader(shader_path: &path::Path, compiler: &mut shaderc::Compiler, shader_type: shaderc::ShaderKind, options: Option<&shaderc::CompileOptions>) {
    let source_text = fs::read_to_string(shader_path).unwrap();

    let spirv = compiler.compile_into_spirv(
        &source_text, 
        shader_type, 
        shader_path.file_name().unwrap().to_str().unwrap(), 
        "main", 
        options,
    ).unwrap();

    let target_path = shader_path.parent().unwrap();

    let output_path = target_path.join(format!("{}.spv", shader_path.file_name().unwrap().to_str().unwrap()));
    let mut output = fs::File::create(output_path).unwrap();
    output.write_all(spirv.as_binary_u8()).unwrap();
}

fn compile_shaders<P: AsRef<path::Path>>(path: P, compiler: &mut shaderc::Compiler) {
    for entry in fs::read_dir(path).unwrap() {
        let entry_path = entry.unwrap().path();

        if entry_path.is_dir() {
            compile_shaders(entry_path, compiler);
        } else {
            let file_extension = entry_path.extension().unwrap().to_str().unwrap();

            // Generate spirv and save
            match file_extension {
                "hlsl" => {
                    let mut options = shaderc::CompileOptions::new().unwrap();
                    options.set_source_language(shaderc::SourceLanguage::HLSL);

                    let file_name = entry_path.file_name().unwrap().to_str().unwrap();

                    let shader_type = if file_name.contains("frag") {
                        shaderc::ShaderKind::Fragment
                    } else if file_name.contains("vert") {
                        shaderc::ShaderKind::Vertex
                    } else {
                        panic!("Could not determine hlsl shader type. Put 'frag' or 'vert' somewhere in the file name: {:?}", entry_path);
                    };

                    compile_and_save_shader(entry_path.as_path(), compiler, shader_type, Some(&options));
                }

                "spv" => {
                    // Nothing to do
                }

                "frag" => {
                    compile_and_save_shader(entry_path.as_path(), compiler, shaderc::ShaderKind::Fragment, None);
                }

                "vert" => {
                    compile_and_save_shader(entry_path.as_path(), compiler, shaderc::ShaderKind::Vertex, None);
                }

                "comp" => {
                    compile_and_save_shader(entry_path.as_path(), compiler, shaderc::ShaderKind::Compute, None);
                }

                _ => {
                    // panic!("Unrecognized shader type at {:?}", entry_path);
                }
            }
        }
    }
}