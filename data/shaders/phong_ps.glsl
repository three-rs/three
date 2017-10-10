#version 150 core
#include <locals>
#include <lights>
#include <globals>

in vec3 v_World;
in vec3 v_Normal;
in vec3 v_Half[MAX_LIGHTS];
in vec4 v_ShadowCoord[MAX_LIGHTS];

out vec4 Target0;

uniform sampler2DShadow t_Shadow0;
uniform sampler2DShadow t_Shadow1;

void main() {
    vec4 color = vec4(0.0);
    vec3 normal = normalize(v_Normal);
    float glossiness = u_MatParams.x;
    for(uint i=0U; i < min(MAX_LIGHTS, u_NumLights); ++i) {
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
            color += shadow * light.intensity.x * u_Color * irradiance;
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
