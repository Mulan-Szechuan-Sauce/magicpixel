#version 150

layout(pixel_center_integer) in vec4 gl_FragCoord;
out vec4 colorOut;

uniform float scale;
uniform int grid_width;
uniform int win_height;

// FIXME: Complete hack, use a buffer texture to allow bigger grids
uniform float[16000] particles;
 
void main() {
    float x = gl_FragCoord.x;
    //float y = float(win_height) - gl_FragCoord.y;
    float y = float(win_height) - gl_FragCoord.y;

    int grid_x = int(x / scale);
    int grid_y = int(y / scale);

    float val = particles[grid_x + grid_y * grid_width];

    //colorOut = vec4(0.0, y / float(win_height), 0.0, 1.0);
    if (val == 1.0) {
        colorOut = vec4(0.0, 0.0, 1.0, 1.0);
    } else {
        colorOut = vec4(0.0, 0.0, 0.0, 1.0);
    }
}
