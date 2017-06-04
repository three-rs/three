#version 150 core
in vec4 a_Position;
in vec4 a_Normal;
uniform b_Globals {
    mat4 u_ViewProj;
};
uniform b_Locals {
    mat4 u_World;
    vec4 u_Color;
    vec4 u_MatParams;
    vec4 u_UvRange;
};
void main() {
    gl_Position = u_ViewProj * u_World * a_Position;
}
