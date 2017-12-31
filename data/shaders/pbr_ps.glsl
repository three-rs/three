
// Original source from https://github.com/KhronosGroup/glTF-WebGL-PBR.
//
// Copyright (c) 2016-2017 Mohamad Moneimne and Contributors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in the
// Software without restriction, including without limitation the rights to use,
// copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the
// Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

#version 150 core
#include <lights>
#include <globals>

const int BASE_COLOR_MAP          = 1 << 0;
const int NORMAL_MAP              = 1 << 1;
const int METALLIC_ROUGHNESS_MAP  = 1 << 2;
const int EMISSIVE_MAP            = 1 << 3;
const int OCCLUSION_MAP           = 1 << 4;
const int DISPLACEMENT_BUFFER     = 1 << 5;

uniform sampler2D u_BaseColorSampler;
uniform sampler2D u_NormalSampler;
uniform sampler2D u_EmissiveSampler;
uniform sampler2D u_MetallicRoughnessSampler;
uniform sampler2D u_OcclusionSampler;

layout(std140) uniform b_PbrParams {
    vec4 u_BaseColorFactor;
    vec3 u_Camera;
    vec3 u_EmissiveFactor;
    vec2 u_MetallicRoughnessValues;
    float u_NormalScale;
    float u_OcclusionStrength;
    int u_PbrFlags;
};

in vec3 v_Position;
in vec2 v_TexCoord;
in mat3 v_Tbn;

out vec4 Target0;

struct PbrInfo {
    float ndotl;
    float ndotv;
    float ndoth;
    float ldoth;
    float vdoth;
    float perceptual_roughness;
    float metalness;
    vec3 base_color;
    vec3 reflectance0;
    vec3 reflectance90;
    float alpha_roughness;
};

const float PI = 3.141592653589793;
const float MIN_ROUGHNESS = 0.04;

float smith(float ndotv, float r)
{
    float tan_sq = (1.0 - ndotv * ndotv) / max((ndotv * ndotv), 0.00001);
    return 2.0 / (1.0 + sqrt(1.0 + r * r * tan_sq));
}

float geometric_occlusion_smith_ggx(PbrInfo pbr)
{
    return smith(pbr.ndotl, pbr.alpha_roughness) * smith(pbr.ndotv, pbr.alpha_roughness);
}

// Basic Lambertian diffuse, implementation from Lambert's Photometria
// https://archive.org/details/lambertsphotome00lambgoog
vec3 lambertian_diffuse(PbrInfo pbr)
{
    return pbr.base_color / PI;
}

// The following equations model the Fresnel reflectance term of the spec equation
// (aka F()) implementation of fresnel from “An Inexpensive BRDF Model for Physically
// based Rendering” by Christophe Schlick
vec3 fresnel_schlick(PbrInfo pbr)
{
    return pbr.reflectance0 + (pbr.reflectance90 - pbr.reflectance0) * pow(clamp(1.0 - pbr.vdoth, 0.0, 1.0), 5.0);
}

// The following equation(s) model the distribution of microfacet normals across
// the area being drawn (aka D())
// Implementation from “Average Irregularity Representation of a Roughened Surface
// for Ray Reflection” by T. S. Trowbridge, and K. P. Reitz
float ggx(PbrInfo pbr)
{
    float roughness_sq = pbr.alpha_roughness * pbr.alpha_roughness;
    float f = (pbr.ndoth * roughness_sq - pbr.ndoth) * pbr.ndoth + 1.0;
    return roughness_sq / (PI * f * f);
}

bool available(int flag)
{
    return (u_PbrFlags & flag) == flag;
}

void main()
{
    mat3 tbn = v_Tbn;
    vec3 v = normalize(u_Camera - v_Position);

    vec3 n;
    if (available(NORMAL_MAP)) {
        n = texture(u_NormalSampler, v_TexCoord).rgb;
        n = normalize(tbn * ((2.0 * n - 1.0) * vec3(u_NormalScale, u_NormalScale, 1.0)));
    } else {
        n = tbn[2].xyz;
    }

    float perceptual_roughness = u_MetallicRoughnessValues.y;
    float metallic = u_MetallicRoughnessValues.x;

    if (available(METALLIC_ROUGHNESS_MAP)) {
	vec4 mr_sample = texture(u_MetallicRoughnessSampler, v_TexCoord);
	perceptual_roughness = mr_sample.g * perceptual_roughness;
	metallic = mr_sample.b * metallic;
    }

    perceptual_roughness = clamp(perceptual_roughness, MIN_ROUGHNESS, 1.0);
    metallic = clamp(metallic, 0.0, 1.0);

    vec4 base_color;
    if (available(BASE_COLOR_MAP)) {
	base_color = texture(u_BaseColorSampler, v_TexCoord) * u_BaseColorFactor;
    } else {
	base_color = u_BaseColorFactor;
    }

    vec3 f0 = vec3(0.04);
    vec3 diffuse_color = mix(base_color.rgb * (1.0 - f0), vec3(0.0, 0.0, 0.0), metallic);
    vec3 specular_color = mix(f0, base_color.rgb, metallic);
    float reflectance = max(max(specular_color.r, specular_color.g), specular_color.b);

    // For typical incident reflectance range (between 4% to 100%) set the grazing
    // reflectance to 100% for typical fresnel effect.
    // For very low reflectance range on highly diffuse objects (below 4%),
    // incrementally reduce grazing reflecance to 0%.
    float reflectance90 = clamp(reflectance * 25.0, 0.0, 1.0);
    vec3 specular_environment_r0 = specular_color.rgb;
    vec3 specular_environment_r90 = vec3(1.0, 1.0, 1.0) * reflectance90;

    // Roughness is authored as perceptual roughness; as is convention, convert to
    // material roughness by squaring the perceptual roughness
    float alpha_roughness = perceptual_roughness * perceptual_roughness;

    vec3 color = vec3(0.0);
    for (uint i = 0U; i < min(MAX_LIGHTS, u_NumLights); ++i) {
	Light light = u_Lights[i];
	vec3 l = normalize(light.dir.xyz);
	vec3 h = normalize(l + v);
	vec3 reflection = -normalize(reflect(v, n));

	float ndotl = clamp(dot(n, l), 0.001, 1.0);
	float ndotv = abs(dot(n, v)) + 0.001;
	float ndoth = clamp(dot(n, h), 0.0, 1.0);
	float ldoth = clamp(dot(l, h), 0.0, 1.0);
	float vdoth = clamp(dot(v, h), 0.0, 1.0);
	PbrInfo pbr_inputs = PbrInfo(
	    ndotl,
	    ndotv,
	    ndoth,
	    ldoth,
	    vdoth,
	    perceptual_roughness,
	    metallic,
	    diffuse_color,
	    specular_environment_r0,
	    specular_environment_r90,
	    alpha_roughness
	);
	vec3 f = fresnel_schlick(pbr_inputs);
	float g = geometric_occlusion_smith_ggx(pbr_inputs);
	float d = ggx(pbr_inputs);
	vec3 diffuse_contrib = (1.0 - f) * lambertian_diffuse(pbr_inputs);
	vec3 spec_contrib = f * g * d / (4.0 * ndotl * ndotv);
	color += ndotl * light.intensity.y * light.color.rgb * (diffuse_contrib + spec_contrib);
    }

    if (available(OCCLUSION_MAP)) {
	float ao = texture(u_OcclusionSampler, v_TexCoord).r;
        color = mix(color, color * ao, u_OcclusionStrength);
    }

    if (available(EMISSIVE_MAP)) {
        vec3 emissive = texture(u_EmissiveSampler, v_TexCoord).rgb * u_EmissiveFactor;
        color += emissive;
    }

    Target0 = vec4(color, base_color.a);
}
