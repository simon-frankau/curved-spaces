uniform float tilt;
uniform float turn;

in vec2 in_vert;
out vec2 vert;

void main() {
    const mat4 projection = mat4(
        vec4(1.0, 0.0, 0.0, 0.0),
        vec4(0.0, 1.0, 0.0, 0.0),
        vec4(0.0, 0.0, 0.5, 0.5),
        vec4(0.0, 0.0, 0.0, 1.0)
    );

    mat4 rot_tilt = mat4(
        vec4(1.0,        0.0,        0.0, 0.0),
        vec4(0.0,  cos(tilt),  sin(tilt), 0.0),
        vec4(0.0, -sin(tilt),  cos(tilt), 0.0),
	vec4(0.0,        0.0,        0.0, 1.0)
    );

    mat4 rot_turn = mat4(
        vec4(cos(turn),  0.0,  sin(turn), 0.0),
        vec4(      0.0,  1.0,        0.0, 0.0),
        vec4(sin(turn),  0.0, -cos(turn), 0.0),
	vec4(0.0,        0.0,        0.0, 1.0)
    );

    gl_Position = (projection * rot_tilt * rot_turn * vec4(in_vert - 0.5, 0, 1));
    vert = gl_Position.xy;
}
