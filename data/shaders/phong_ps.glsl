#version 150 core

in vec3 v_World;
in vec3 v_Normal;
in vec3 v_Half[4];
in vec4 v_ShadowCoord[4];

out vec4 Target0;

uniform sampler2DShadow t_Shadow0;
uniform sampler2DShadow t_Shadow1;

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
    vec4 color = vec4(0.0);
    vec3 normal = normalize(v_Normal);
    float glossiness = u_MatParams.x;
    for(uint i=0U; i<4U && i < u_NumLights; ++i) {
        Light light = u_Lights[i];
        vec4 lit_space = v_ShadowCoord[i];
        float shadow = 1.0;
        if (light.shadow_params[0] == 0) {
            shadow = texture(t_Shadow0, 0.5 * lit_space.xyz / lit_space.w + 0.5);
        }
        if (light.shadow_params[0] == 1) {
            shadow = texture(t_Shadow1, 0.5 * lit_space.xyz / lit_space.w + 0.5);
        }
        if (shadow == 0.0) {
            continue;
        }
        vec3 dir = light.pos.xyz - light.pos.w * v_World.xyz;
        float dot_nl = dot(normal, normalize(dir));
        // hemisphere light test
        if (dot(light.color_back, light.color_back) > 0.0) {
            vec4 irradiance = mix(light.color_back, light.color, dot_nl*0.5 + 0.5);
            color += shadow * light.intensity.y * u_Color * irradiance;
        } else {
            float kd = light.intensity.x + light.intensity.y * max(0.0, dot_nl);
            color += shadow * kd * u_Color * light.color;
        }
        if (dot_nl > 0.0 && glossiness > 0.0) {
            float ks = dot(normal, normalize(v_Half[i]));
            if (ks > 0.0) {
                color += shadow * pow(ks, glossiness) * light.color;
            }
        }
    }
    Target0 = color;
}
