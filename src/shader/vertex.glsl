#define M_PI 3.1415926535897932384626433832795

// Radius of sphere containing image, pre-transform.
//
// The display fits in a -1..1 cube, but may be rotated, so
// absolute maximum radius is sqrt(3). Let's try sqrt(2) as
// more of a compromise for what real usage is like (to avoid
// zooming out too much).
const float r1 = sqrt(2.0);
// Radius of sphere containing image, post-transform.
const float r2 = sqrt(2.0) + 1.0;
// Scaling factor.
const float s = r2 / r1;

uniform float x_scale;
uniform float y_scale;

uniform float tilt;
uniform float turn;

uniform vec3 color;

in vec3 in_vert;

out vec3 base_color;

void main() {
    float tilt_rad = -tilt * M_PI / 180.0;
    float turn_rad =  turn * M_PI / 180.0;

    // Rotate the image in the XZ plane (around Y axis).
    //
    // We also flip the Z axis, in combination with the xzy swizzle
    // below, to get the grid around the way we want it.
    mat4 rot_turn = mat4(
        vec4( cos(turn_rad),  0.0,  sin(turn_rad), 0.0),
        vec4(           0.0,  1.0,            0.0, 0.0),
        vec4(-sin(turn_rad),  0.0,  cos(turn_rad), 0.0),
        vec4(           0.0,  0.0,            0.0, 1.0)
    );

    // Tilt the image in the YZ plane (around X axis).
    mat4 rot_tilt = mat4(
        vec4(1.0,            0.0,            0.0, 0.0),
        vec4(0.0,  cos(tilt_rad),  sin(tilt_rad), 0.0),
        vec4(0.0, -sin(tilt_rad),  cos(tilt_rad), 0.0),
        vec4(0.0,            0.0,            0.0, 1.0)
    );

    // Translate and scale the image so that it's beyond the clip
    // plane, but should touch the edge of screen at maximum extent.
    mat4 scale = mat4(
         vec4(  s, 0.0, 0.0, 0.0),
         vec4(0.0,   s, 0.0, 0.0),
         vec4(0.0, 0.0,   s, 0.0),
         vec4(0.0, 0.0,  r2, 1.0)
    );

    // NB: Transposed because OpenGL likes constructing matrices that
    // way. I'm sure this is well known to OpenGL people, but surprised a
    // newbie like me.
    mat4 projection = mat4(
        vec4(x_scale,     0.0, 0.0, 0.0),
        vec4(    0.0, y_scale, 0.0, 0.0),
        vec4(    0.0,     0.0, 1.0, 1.0),
        vec4(    0.0,     0.0, 0.0, 1.0)
    );

    gl_Position = (projection * scale * rot_tilt * rot_turn * vec4(in_vert.xzy, 1));

    base_color = color;
}
