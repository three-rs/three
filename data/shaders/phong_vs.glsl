#version 150 core
in vec4 a_Position;
in vec4 a_Normal;
out vec3 v_World;
out vec3 v_Normal;
out vec3 v_Half[4];
out vec4 v_ShadowCoord[4];
struct Light {
    mat4 projection;
    vec4 pos;
    vec4 dir;
    vec4 focus;
    vec4 color;
    vec4 color_back;
    vec4 intensity;
    ivec4 shadow_params;
};
uniform b_Lights {
    Light u_Lights[4];
};
uniform b_Globals {
    mat4 u_ViewProj;
    uint u_NumLights;
};
uniform b_Locals {
    mat4 u_World;
    vec4 u_Color;
    vec4 u_MatParams;
    vec4 u_UvRange;
};
void main() {
    vec4 world = u_World * a_Position;
    v_World = world.xyz;
    v_Normal = normalize(mat3(u_World) * a_Normal.xyz);
    for(uint i=0U; i<4U && i < u_NumLights; ++i) {
        Light light = u_Lights[i];
        vec3 dir = light.pos.xyz - light.pos.w * world.xyz;
        v_Half[i] = normalize(v_Normal + normalize(dir));
        v_ShadowCoord[i] = light.projection * world;
    }
    gl_Position = u_ViewProj * world;
}
