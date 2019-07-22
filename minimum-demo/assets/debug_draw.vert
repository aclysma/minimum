#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(set = 0, binding = 0) uniform Args {
    mat4 mvp;
} uniform_buffer;

layout(location = 0) in vec3 pos;
layout(location = 1) in vec4 in_color;
layout(location = 0) out vec4 color;

void main() {
    color = in_color;
    gl_Position = uniform_buffer.mvp * vec4(pos.x, pos.y, pos.z, 1.0);
}
