#version 150 core
#include <globals>

in vec4 a_Position;
in vec4 a_Normal;
in vec2 a_TexCoord;
out vec2 v_TexCoord;
out vec4 v_Color;

in vec4 i_World0;
in vec4 i_World1;
in vec4 i_World2;
in vec4 i_World3;
in vec4 i_Color;
in vec4 i_UvRange;

void main() {
    mat4 m_World = mat4(i_World0, i_World1, i_World2, i_World3);
    v_TexCoord = mix(i_UvRange.xy, i_UvRange.zw, a_TexCoord);
    v_Color = i_Color;
    gl_Position = u_ViewProj * m_World * a_Position;
}
