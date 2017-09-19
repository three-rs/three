#version 150 core
#include globals

out vec3 v_TexCoord;

void main() {
    vec2 pos = gl_VertexID == 0 ? vec2(-1.0, -1.0) :
               gl_VertexID == 1 ? vec2(-1.0,  1.0) :
               gl_VertexID == 3 ? vec2( 1.0,  1.0) :
                                  vec2( 1.0, -1.0) ;

    vec4 a_Position = vec4(pos.xy, 1.0, 1.0);

    mat3 inverseView = transpose(mat3(u_View));
    vec3 unprojected = (u_InverseProj * a_Position).xyz;

    v_TexCoord = inverseView * unprojected;

    gl_Position = a_Position;
}
