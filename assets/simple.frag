#version 150

layout(pixel_center_integer) in vec4 gl_FragCoord;
out vec4 colorOut;

void main() {
    float x = int(gl_FragCoord.x);
    float y = int(gl_FragCoord.y);

    colorOut = vec4(0.0, 0.0, 1.0, 1.0);
}
