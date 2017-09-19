#version 150 core

uniform samplerCube t_Input;

in vec3 v_TexCoord;
out vec4 Target0;

void main() {
    Target0 = texture(t_Input, v_TexCoord);
}
