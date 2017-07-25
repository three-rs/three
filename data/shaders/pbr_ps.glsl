
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

#version 150
#extension GL_EXT_shader_texture_lod: enable
#extension GL_OES_standard_derivatives: enable

const int PBR_FLAG_HAS_BASE_COLOR_MAP          = 1 << 0;
const int PBR_FLAG_HAS_NORMAL_MAP              = 1 << 1;
const int PBR_FLAG_HAS_METALLIC_ROUGHNESS_MAP  = 1 << 2;
const int PBR_FLAG_HAS_EMISSIVE_MAP            = 1 << 3;
const int PBR_FLAG_HAS_OCCLUSION_MAP           = 1 << 4;

uniform sampler2D u_BaseColorSampler;
uniform sampler2D u_NormalSampler;
uniform sampler2D u_EmissiveSampler;
uniform sampler2D u_MetallicRoughnessSampler;
uniform sampler2D u_OcclusionSampler;

uniform b_PbrParams {
    vec4 u_BaseColorFactor;
    vec3 u_Camera;
    vec3 u_LightDirection;
    vec3 u_LightColor;
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
    float NdotL;
    float NdotV;
    float NdotH;
    float LdotH;
    float VdotH;
    float perceptualRoughness;
    float metalness;
    vec3 baseColor;
    vec3 reflectance0;
    vec3 reflectance90;
    float alphaRoughness;
};

const float M_PI = 3.141592653589793;
const float c_MinRoughness = 0.04;

// The following equations model the diffuse term of the lighting equation
// Implementation of diffuse from "Physically-Based Shading at Disney" by Brent Burley
vec3 disneyDiffuse(PbrInfo pbrInputs)
{
    float f90 = 2.0 * pbrInputs.LdotH * pbrInputs.LdotH * pbrInputs.alphaRoughness - 0.5;

    return (pbrInputs.baseColor / M_PI) * (1.0 + f90 * pow((1.0 - pbrInputs.NdotL), 5.0)) * (1.0 + f90 * pow((1.0 - pbrInputs.NdotV), 5.0));
}

// basic Lambertian diffuse, implementation from Lambert's Photometria https://archive.org/details/lambertsphotome00lambgoog
vec3 lambertianDiffuse(PbrInfo pbrInputs)
{
    return pbrInputs.baseColor / M_PI;
}

// The following equations model the Fresnel reflectance term of the spec equation (aka F())
// implementation of fresnel from “An Inexpensive BRDF Model for Physically based Rendering” by Christophe Schlick
vec3 fresnelSchlick2(PbrInfo pbrInputs)
{
    return pbrInputs.reflectance0 + (pbrInputs.reflectance90 - pbrInputs.reflectance0) * pow(clamp(1.0 - pbrInputs.VdotH, 0.0, 1.0), 5.0);
}

// Simplified implementation of fresnel from “An Inexpensive BRDF Model for Physically based Rendering” by Christophe Schlick
vec3 fresnelSchlick(PbrInfo pbrInputs)
{
    return pbrInputs.metalness + (vec3(1.0) - pbrInputs.metalness) * pow(1.0 - pbrInputs.VdotH, 5.0);
}

// The following equations model the geometric occlusion term of the spec equation  (aka G())
// Implementation from “A Reflectance Model for Computer Graphics” by Robert Cook and Kenneth Torrance,
float geometricOcclusionCookTorrance(PbrInfo pbrInputs)
{
    return min(min(2.0 * pbrInputs.NdotV * pbrInputs.NdotH / pbrInputs.VdotH, 2.0 * pbrInputs.NdotL * pbrInputs.NdotH / pbrInputs.VdotH), 1.0);
}

// implementation of microfacet occlusion from “An Inexpensive BRDF Model for Physically based Rendering” by Christophe Schlick
float geometricOcclusionSchlick(PbrInfo pbrInputs)
{
    float k = pbrInputs.perceptualRoughness * 0.79788; // 0.79788 = sqrt(2.0/3.1415); perceptualRoughness = sqrt(alphaRoughness);
  // alternately, k can be defined with
  // float k = (pbrInputs.perceptualRoughness + 1) * (pbrInputs.perceptualRoughness + 1) / 8;

    float l = pbrInputs.LdotH / (pbrInputs.LdotH * (1.0 - k) + k);
    float n = pbrInputs.NdotH / (pbrInputs.NdotH * (1.0 - k) + k);
    return l * n;
}

// the following Smith implementations are from “Geometrical Shadowing of a Random Rough Surface” by Bruce G. Smith
float geometricOcclusionSmith(PbrInfo pbrInputs)
{
    float NdotL2 = pbrInputs.NdotL * pbrInputs.NdotL;
    float NdotV2 = pbrInputs.NdotV * pbrInputs.NdotV;
    float v = ( -1.0 + sqrt ( pbrInputs.alphaRoughness * (1.0 - NdotL2 ) / NdotL2 + 1.)) * 0.5;
    float l = ( -1.0 + sqrt ( pbrInputs.alphaRoughness * (1.0 - NdotV2 ) / NdotV2 + 1.)) * 0.5;
    return (1.0 / max((1.0 + v + l ), 0.000001));
}

float SmithG1_var2(float NdotV, float r)
{
    float tanSquared = (1.0 - NdotV * NdotV) / max((NdotV * NdotV), 0.00001);
    return 2.0 / (1.0 + sqrt(1.0 + r * r * tanSquared));
}

float SmithG1(float NdotV, float r)
{
    return 2.0 * NdotV / (NdotV + sqrt(r * r + (1.0 - r * r) * (NdotV * NdotV)));
}

float geometricOcclusionSmithGGX(PbrInfo pbrInputs)
{
    return SmithG1_var2(pbrInputs.NdotL, pbrInputs.alphaRoughness) * SmithG1_var2(pbrInputs.NdotV, pbrInputs.alphaRoughness);
}

// The following equation(s) model the distribution of microfacet normals across the area being drawn (aka D())
// implementation from “Average Irregularity Representation of a Roughened Surface for Ray Reflection” by T. S. Trowbridge, and K. P. Reitz
float GGX(PbrInfo pbrInputs)
{
    float roughnessSq = pbrInputs.alphaRoughness * pbrInputs.alphaRoughness;
    float f = (pbrInputs.NdotH * roughnessSq - pbrInputs.NdotH) * pbrInputs.NdotH + 1.0;
    return roughnessSq / (M_PI * f * f);
}

bool has_flag(int flag)
{
    return (u_PbrFlags & flag) == flag;
}

void main()
{
    mat3 tbn = v_Tbn;

    vec3 n;
    if (has_flag(PBR_FLAG_HAS_NORMAL_MAP)) {
        n = texture2D(u_NormalSampler, v_TexCoord).rgb;
        n = normalize(tbn * ((2.0 * n - 1.0) * vec3(u_NormalScale, u_NormalScale, 1.0)));
    } else {
      n = tbn[2].xyz;
    }

    vec3 v = normalize(u_Camera - v_Position);
    vec3 l = normalize(u_LightDirection);
    vec3 h = normalize(l + v);
    vec3 reflection = -normalize(reflect(v, n));

    float NdotL = clamp(dot(n, l), 0.001, 1.0);
    float NdotV = abs(dot(n, v)) + 0.001;
    float NdotH = clamp(dot(n, h), 0.0, 1.0);
    float LdotH = clamp(dot(l, h), 0.0, 1.0);
    float VdotH = clamp(dot(v, h), 0.0, 1.0);

    float perceptualRoughness = u_MetallicRoughnessValues.y;
    float metallic = u_MetallicRoughnessValues.x;

    if (has_flag(PBR_FLAG_HAS_METALLIC_ROUGHNESS_MAP)) {
        vec4 mrSample = texture2D(u_MetallicRoughnessSampler, v_TexCoord);
        perceptualRoughness = mrSample.g * perceptualRoughness;
        metallic = mrSample.b * metallic;
    }

    perceptualRoughness = clamp(perceptualRoughness, c_MinRoughness, 1.0);
    metallic = clamp(metallic, 0.0, 1.0);

    vec4 baseColor;
    if (has_flag(PBR_FLAG_HAS_BASE_COLOR_MAP)) {
        baseColor = texture2D(u_BaseColorSampler, v_TexCoord) * u_BaseColorFactor;
    } else {
        baseColor = u_BaseColorFactor;
    }
    
    vec3 f0 = vec3(0.04);
    // is this the same? test!
    vec3 diffuseColor = mix(baseColor.rgb * (1.0 - f0), vec3(0.0, 0.0, 0.0), metallic);
    //vec3 diffuseColor = baseColor * (1.0 - metallic);
    vec3 specularColor = mix(f0, baseColor.rgb, metallic);

    // Compute reflectance.
    float reflectance = max(max(specularColor.r, specularColor.g), specularColor.b);

    // For typical incident reflectance range (between 4% to 100%) set the grazing reflectance to 100% for typical fresnel effect.
    // For very low reflectance range on highly diffuse objects (below 4%), incrementally reduce grazing reflecance to 0%.
    float reflectance90 = clamp(reflectance * 25.0, 0.0, 1.0);
    vec3 specularEnvironmentR0 = specularColor.rgb;
    vec3 specularEnvironmentR90 = vec3(1.0, 1.0, 1.0) * reflectance90;

    // roughness is authored as perceptual roughness; as is convention, convert to material roughness by squaring the perceptual roughness
    float alphaRoughness = perceptualRoughness * perceptualRoughness;

    PbrInfo pbrInputs = PbrInfo(
	NdotL,
	NdotV,
	NdotH,
	LdotH,
	VdotH,
	perceptualRoughness,
	metallic,
	diffuseColor,
	specularEnvironmentR0,
	specularEnvironmentR90,
	alphaRoughness
    );
    vec3 F = fresnelSchlick2(pbrInputs);
    //vec3 F = fresnelSchlick(pbrInputs);
    //float G = geometricOcclusionCookTorrance(pbrInputs);
    //float G = geometricOcclusionSmith(pbrInputs);
    //float G = geometricOcclusionSchlick(pbrInputs);
    float G = geometricOcclusionSmithGGX(pbrInputs);
    float D = GGX(pbrInputs);
    vec3 diffuseContrib = (1.0 - F) * lambertianDiffuse(pbrInputs);
    //vec3 diffuseContrib = (1.0 - F) * disneyDiffuse(pbrInputs);
    vec3 specContrib = F * G * D / (4.0 * NdotL * NdotV);
    vec3 color = NdotL * u_LightColor * (diffuseContrib + specContrib);

    if (has_flag(PBR_FLAG_HAS_OCCLUSION_MAP)) {
        float ao = texture2D(u_OcclusionSampler, v_TexCoord).r;
        color = mix(color, color * ao, u_OcclusionStrength);
    }

    if (has_flag(PBR_FLAG_HAS_EMISSIVE_MAP)) {
        vec3 emissive = texture2D(u_EmissiveSampler, v_TexCoord).rgb * u_EmissiveFactor;
        color += emissive;
    }

    Target0 = vec4(color, baseColor.a);
}
