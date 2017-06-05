#version 150 core

in vec4 v_ResultColor;
in vec4 v_LightEval[2];
in vec4 v_ShadowCoord[2];

out vec4 Target0;

uniform sampler2DShadow t_Shadow0;
uniform sampler2DShadow t_Shadow1;

void main() {
    Target0 = v_ResultColor;
    if (v_ShadowCoord[0].w != 0.0) {
        vec3 coord = v_ShadowCoord[0].xyz / v_ShadowCoord[0].w;
        float shadow = texture(t_Shadow0, 0.5 * coord + 0.5);
        Target0 += shadow * v_LightEval[0];
    }
    if (v_ShadowCoord[1].w != 0.0) {
        vec3 coord = v_ShadowCoord[1].xyz / v_ShadowCoord[1].w;
        float shadow = texture(t_Shadow1, 0.5 * coord + 0.5);
        Target0 += shadow * v_LightEval[1];
    }
}
