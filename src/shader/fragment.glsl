precision mediump float;

in vec3 base_color;

out vec4 color;
void main() {
    color = vec4(base_color, 1.0);
}
