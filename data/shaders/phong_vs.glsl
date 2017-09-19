#version 150 core
#include locals lights globals

in vec4 a_Position;
in vec4 a_Normal;
out vec3 v_World;
out vec3 v_Normal;
out vec3 v_Half[MAX_LIGHTS];
out vec4 v_ShadowCoord[MAX_LIGHTS];

void main() {
    vec4 world = u_World * a_Position;
    v_World = world.xyz;
    v_Normal = normalize(mat3(u_World) * a_Normal.xyz);
    for(uint i=0U; i < min(MAX_LIGHTS, u_NumLights); ++i) {
        Light light = u_Lights[i];
        vec3 dir = light.pos.xyz - light.pos.w * world.xyz;
        v_Half[i] = normalize(v_Normal + normalize(dir));
        v_ShadowCoord[i] = light.projection * world;
    }
    gl_Position = u_ViewProj * world;
}
