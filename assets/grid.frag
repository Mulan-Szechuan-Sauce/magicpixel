#version 460 core

layout(std430, binding = 3) buffer layoutName
{
    uint grid[];
};

layout(pixel_center_integer) in vec4 gl_FragCoord;
out vec4 colorOut;

void main() {
    // TODO: Pass these in as uniforms
    int grid_width = 100;
    float scale = 14.0;
    float win_height = 1400.0;
    float max_fill = 64.0;

    float x = gl_FragCoord.x;
    float y = win_height - gl_FragCoord.y;

    int grid_x = int(x / scale);
    int grid_y = int(y / scale);

    uint val = grid[grid_x + grid_y * grid_width];

    //self.pixel_data[i] = ((type_id as u32) << 8) + fill_ratio as u32;

    uint p_type_id  = (val >> 8) & 0xff;
    uint fill_ratio = val & 0xff;

    if (p_type_id == 0) {
        colorOut = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        colorOut = vec4(0.0, 0.0, fill_ratio / max_fill, 1.0);
    }
}
