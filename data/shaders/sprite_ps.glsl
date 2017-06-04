#version 150 core
in vec2 v_TexCoord;
uniform sampler2D t_Map;
void main() {
    gl_FragColor = texture(t_Map, v_TexCoord);
}
