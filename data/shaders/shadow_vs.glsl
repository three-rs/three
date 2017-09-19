#version 150 core
#include locals globals

in vec4 a_Position;

void main() {
    gl_Position = u_ViewProj * u_World * a_Position;
}
