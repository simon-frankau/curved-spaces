in vec2 in_vert;
out vec2 vert;

void main() {
    vert = in_vert;
    gl_Position = vec4(vert - 0.5, 0.0, 1.0);
}
