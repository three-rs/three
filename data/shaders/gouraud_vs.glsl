#version 150 core
#include <lights>
#include <globals>

#define MAX_SHADOWS 2

in vec4 a_Position;
in vec4 a_Normal;
out vec4 v_ResultColor;
flat out vec4 v_ResultColorFlat;
flat out float v_Smooth;
out vec4 v_LightEval[MAX_SHADOWS];
flat out vec4 v_LightEvalFlat[MAX_SHADOWS];
out vec4 v_ShadowCoord[MAX_SHADOWS];

in vec4 i_World0;
in vec4 i_World1;
in vec4 i_World2;
in vec4 i_MatParams;
in vec4 i_Color;
in vec4 i_UvRange;

void main() {
    mat4 m_World = transpose(mat4(i_World0, i_World1, i_World2, vec4(0.0, 0.0, 0.0, 1.0)));
    vec4 world = m_World * a_Position;
    vec3 normal = normalize(mat3(m_World) * a_Normal.xyz);
    for(int i=0; i<MAX_SHADOWS; ++i) {
        v_ShadowCoord[i] = vec4(0.0);
        v_LightEval[i] = v_LightEvalFlat[i] = vec4(0.0);
    }
    v_ResultColor = vec4(0.0);
    v_Smooth = i_MatParams.x;

    for(uint i=0U; i < min(MAX_LIGHTS, u_NumLights); ++i) {
        Light light = u_Lights[i];
        vec3 dir = light.pos.xyz - light.pos.w * world.xyz;
        // evaluate light color
        float dot_nl = dot(normal, normalize(dir));
        vec4 irradiance = light.color;
        if (dot(light.color_back, light.color_back) > 0.0) {
            irradiance = mix(light.color_back, light.color, dot_nl*0.5 + 0.5);
            dot_nl = 0.0;
        }
        v_ResultColor += light.intensity.x * i_Color * irradiance; //ambient
        vec4 color = light.intensity.y * max(0.0, dot_nl) * i_Color * light.color;
        // compute shadow coordinates
        int shadow_index = light.shadow_params[0];
        if (0 <= shadow_index && shadow_index < MAX_SHADOWS) {
            v_ShadowCoord[shadow_index] = light.projection * world;
            v_LightEval[shadow_index] = color;
            v_LightEvalFlat[shadow_index] = color;
        } else {
            v_ResultColor += color;
        }
    }

    v_ResultColorFlat = v_ResultColor;
    gl_Position = u_ViewProj * world;
}
