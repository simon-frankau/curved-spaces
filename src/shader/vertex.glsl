#define M_PI 3.1415926535897932384626433832795

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

    float tilt_rad = tilt * M_PI / 180.0;
    float turn_rad = turn * M_PI / 180.0;

    mat4 rot_tilt = mat4(
        vec4(1.0,            0.0,            0.0, 0.0),
        vec4(0.0,  cos(tilt_rad),  sin(tilt_rad), 0.0),
        vec4(0.0, -sin(tilt_rad),  cos(tilt_rad), 0.0),
        vec4(0.0,            0.0,            0.0, 1.0)
    );

    mat4 rot_turn = mat4(
        vec4(cos(turn_rad),  0.0,  sin(turn_rad), 0.0),
        vec4(          0.0,  1.0,            0.0, 0.0),
        vec4(sin(turn_rad),  0.0, -cos(turn_rad), 0.0),
        vec4(0.0,            0.0,            0.0, 1.0)
    );

    gl_Position = (projection * rot_tilt * rot_turn * vec4(in_vert - 0.5, 0, 1));
    vert = gl_Position.xy;
}
