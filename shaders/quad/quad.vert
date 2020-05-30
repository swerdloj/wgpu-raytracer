#version 450

layout(location = 0) in vec3 attr_position;
layout(location = 1) in vec2 attr_tex_coord;

layout(location = 0) out vec2 v_tex_coord;

void main() {
    v_tex_coord = attr_tex_coord;

    gl_Position = vec4(attr_position, 1.);
}