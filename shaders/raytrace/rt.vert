#version 450

void main() {
    // Two triangles filling the screen
    vec3 full_screen[] = vec3[] (
        vec3( 1.0,  1.0, 0.0),
        vec3(-1.0, -1.0, 0.0),
        vec3(-1.0,  1.0, 0.0),

        vec3( 1.0,  1.0, 0.0),
        vec3(-1.0, -1.0, 0.0),
        vec3( 1.0, -1.0, 0.0)
    );
    
    gl_Position = vec4(full_screen[gl_VertexIndex], 1.);
}