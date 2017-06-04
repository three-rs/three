#version 150 core
#include locals

in vec4 a_Position;
uniform b_Globals {
    mat4 u_ViewProj;
};

void main() {
    gl_Position = u_ViewProj * u_World * a_Position;
}
