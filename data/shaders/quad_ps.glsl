#version 150 core
in vec2 v_TexCoord;
uniform sampler2D t_Input;
void main() {
    gl_FragColor = texture(t_Input, v_TexCoord);
}
