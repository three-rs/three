#version 150 core
#include <globals>

in vec4 a_Position;
in vec4 i_World0;
in vec4 i_World1;
in vec4 i_World2;

void main() {
    mat4 m_World = transpose(mat4(i_World0, i_World1, i_World2, vec4(0.0, 0.0, 0.0, 1.0)));
    gl_Position = u_ViewProj * m_World * a_Position;
}
