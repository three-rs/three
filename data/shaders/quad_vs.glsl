#version 150 core

out vec2 v_TexCoord;

uniform b_Params {
    vec4 u_Rect;
    float u_Depth;
};

void main() {
    v_TexCoord = gl_VertexID==0 ? vec2(1.0, 0.0) :
                 gl_VertexID==1 ? vec2(0.0, 0.0) :
                 gl_VertexID==2 ? vec2(1.0, 1.0) :
                                  vec2(0.0, 1.0) ;
    vec2 pos = mix(u_Rect.xy, u_Rect.zw, v_TexCoord);
    gl_Position = vec4(pos, u_Depth, 1.0);
}
