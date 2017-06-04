#version 150 core
uniform b_Locals {
    mat4 u_World;
    vec4 u_Color;
    vec4 u_MatParams;
    vec4 u_UvRange;
};
void main() {
    gl_FragColor = u_Color;
}
