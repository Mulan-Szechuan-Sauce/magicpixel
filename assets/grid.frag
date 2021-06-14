#version 460 core

layout(std430, binding = 3) buffer layoutName
{
    uint grid[];
};

layout(pixel_center_integer) in vec4 gl_FragCoord;
out vec4 colorOut;

uniform int grid_width;
uniform int win_height = 1400;
uniform float scale = 14.0;
uniform int max_fill = 64;

void main() {
    float x = gl_FragCoord.x;
    float y = float(win_height) - gl_FragCoord.y;

    int grid_x = int(x / scale);
    int grid_y = int(y / scale);

    uint val = grid[grid_x + grid_y * grid_width];

    uint p_type_id  = (val >> 8) & 0xff;
    uint fill_ratio = val & 0xff;

    float fill_percent = fill_ratio / float(max_fill);

    if (p_type_id == 1) {
        colorOut = fill_percent * vec4(0.0, 0.0, 1.0, 1.0);
    } else if (p_type_id == 2) {
        colorOut = fill_percent * vec4(194.0/255, 178.0/255, 128.0/255, 1.0);
    } else if (p_type_id == 3) {
        colorOut = fill_percent * vec4(42.0/255, 23.0/255, 11.0/255, 1.0);
    } else {
        colorOut = vec4(0.0, 0.0, 0.0, 1.0);
    }
}
