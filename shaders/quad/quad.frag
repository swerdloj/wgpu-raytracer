#version 450

layout(location = 0) in vec2 v_tex_coords;

layout(set = 0, binding = 0) uniform texture2D u_texture;
layout(set = 0, binding = 1) uniform sampler u_sampler;


layout(location = 0) out vec4 out_color;

void main() {
    vec4 color = texture(sampler2D(u_texture, u_sampler), v_tex_coords);

    out_color = vec4(0., 0., 0., 1.);
}