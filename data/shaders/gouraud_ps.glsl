#version 150 core

in vec4 v_ResultColor;
flat in vec4 v_ResultColorFlat;
flat in float v_Smooth;
in vec4 v_LightEval[2];
flat in vec4 v_LightEvalFlat[2];
in vec4 v_ShadowCoord[2];

out vec4 Target0;

uniform sampler2DShadow t_Shadow0;
uniform sampler2DShadow t_Shadow1;

void main() {
    Target0 = mix(v_ResultColorFlat, v_ResultColor, v_Smooth);
    if (v_ShadowCoord[0].w != 0.0) {
        vec3 coord = v_ShadowCoord[0].xyz / v_ShadowCoord[0].w;
        float shadow = texture(t_Shadow0, 0.5 * coord + 0.5);
        Target0 += shadow * mix(v_LightEvalFlat[0], v_LightEval[0], v_Smooth);
    }
    if (v_ShadowCoord[1].w != 0.0) {
        vec3 coord = v_ShadowCoord[1].xyz / v_ShadowCoord[1].w;
        float shadow = texture(t_Shadow1, 0.5 * coord + 0.5);
        Target0 += shadow * mix(v_LightEvalFlat[1], v_LightEval[1], v_Smooth);
    }
}
