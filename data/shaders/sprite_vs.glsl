#version 150 core
in vec4 a_Position;
in vec2 a_TexCoord;
out vec2 v_TexCoord;
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
    v_TexCoord = mix(u_UvRange.xy, u_UvRange.zw, a_TexCoord);
    gl_Position = u_ViewProj * u_World * a_Position;
}
