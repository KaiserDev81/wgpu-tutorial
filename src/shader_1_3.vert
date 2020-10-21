// shader.vert
#version 450

// Mala practica porque limita la flexibilidad del renderizado
const vec2 positions[3] = vec2[3](
    vec2(0.0, 0.5),
    vec2(-0.5, -0.5),
    vec2(0.5, -0.5)
);

// gl_Position y gl_VertexIndex son variables built_in the GLSL en esta version
void main() {
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
}