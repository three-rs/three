#version 150 core

out vec4 Target0;

uniform b_Locals {
    mat4 u_World;
    vec4 u_Color;
    vec4 u_MatParams;
    vec4 u_UvRange;
};

void main() {
    Target0 = u_Color;
}
