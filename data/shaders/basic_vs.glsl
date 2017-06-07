#version 150 core
#include locals

in vec4 a_Position;
in vec4 a_Normal;
in vec2 a_TexCoord;
out vec2 v_TexCoord;

uniform b_Globals {
    mat4 u_ViewProj;
};

void main() {
    v_TexCoord = mix(u_UvRange.xy, u_UvRange.zw, a_TexCoord);
    gl_Position = u_ViewProj * u_World * a_Position;
}
