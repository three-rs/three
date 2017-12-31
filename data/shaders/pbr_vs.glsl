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
#define MAX_TARGETS 8U
#include <globals>

const int DISPLACEMENT_BUFFER = 1 << 5;

in vec4 a_Position;
in vec2 a_TexCoord;
in vec4 a_Normal;
in vec4 a_Tangent;
in ivec4 a_JointIndices;
in vec4 a_JointWeights;

out vec3 v_Position;
out vec2 v_TexCoord;
out mat3 v_Tbn;
out vec3 v_Normal;

in vec4 i_World0;
in vec4 i_World1;
in vec4 i_World2;

// Toggles displacement contributions to `a_Position/a_Normal/a_Tangent`.
struct DisplacementContribution {
    // position: 1.0 if morph target weights should influence a_Position
    // normal: 1.0 if morph target weights should influence a_Normal
    // tangent: 1.0 if morph target weights should influence a_Tangent
    // weight: The weight to be applied.
    float position, normal, tangent, weight;
};

layout(std140) uniform b_DisplacementContributions {
    DisplacementContribution u_DisplacementContributions[MAX_TARGETS];
};

layout(std140) uniform b_PbrParams {
    vec4 u_BaseColorFactor;
    vec3 u_Camera;
    vec3 u_EmissiveFactor;
    vec2 u_MetallicRoughnessValues;
    float u_NormalScale;
    float u_OcclusionStrength;
    int u_PbrFlags;
};

uniform samplerBuffer b_JointTransforms;
uniform samplerBuffer b_Displacements;

mat4 fetch_joint_transform(int i)
{
    vec4 col0 = texelFetch(b_JointTransforms, 4 * i);
    vec4 col1 = texelFetch(b_JointTransforms, 4 * i + 1);
    vec4 col2 = texelFetch(b_JointTransforms, 4 * i + 2);
    vec4 col3 = texelFetch(b_JointTransforms, 4 * i + 3);

    return mat4(col0, col1, col2, col3);
}

vec3 fetch_displacement(uint i)
{
    int index = gl_VertexID * int(MAX_TARGETS) + int(i);
    vec4 texel = texelFetch(b_Displacements, index);

    return texel.xyz;
}

mat4 compute_skin_transform()
{
    return
	a_JointWeights.x * fetch_joint_transform(a_JointIndices.x) +
	a_JointWeights.y * fetch_joint_transform(a_JointIndices.y) +
	a_JointWeights.z * fetch_joint_transform(a_JointIndices.z) +
	a_JointWeights.w * fetch_joint_transform(a_JointIndices.w);
}

vec4 compute_local_position()
{
    vec4 position = a_Position;

    for (uint i = 0U; i < MAX_TARGETS; ++i) {
	position.xyz +=
	    u_DisplacementContributions[i].position
	    * u_DisplacementContributions[i].weight
	    * fetch_displacement(i);
    }

    return position;
}

vec3 compute_local_normal()
{
    vec3 normal = a_Normal.xyz;

    for (uint i = 0U; i < MAX_TARGETS; ++i) {
	normal +=
	    u_DisplacementContributions[i].normal
	    * u_DisplacementContributions[i].weight
	    * fetch_displacement(i);
    }

    return normal;
}

vec3 compute_local_tangent()
{
    vec3 tangent = a_Tangent.xyz;

    for (uint i = 0U; i < MAX_TARGETS; ++i) {
	tangent +=
	    u_DisplacementContributions[i].tangent
	    * u_DisplacementContributions[i].weight
	    * fetch_displacement(i);
    }

    return tangent;
}

bool available(int flag)
{
    return (u_PbrFlags & flag) == flag;
}

void main()
{
    mat4 mx_world = transpose(mat4(i_World0, i_World1, i_World2, vec4(0.0, 0.0, 0.0, 1.0)));
    mat4 mx_mvp = u_ViewProj * mx_world;
    mat4 mx_skin = compute_skin_transform();

    vec4 local_position;
    vec3 local_normal;
    vec3 local_tangent;
    if (available(DISPLACEMENT_BUFFER)) {
	local_position = compute_local_position();
	local_normal = compute_local_normal();
	local_tangent = compute_local_tangent();
    } else {
	local_position = a_Position;
	local_normal = a_Normal.xyz;
	local_tangent = a_Tangent.xyz;
    }

    vec4 world_position = mx_world * local_position;
    vec3 world_normal = normalize(vec3(mx_world * vec4(local_normal, 0.0)));
    vec3 world_tangent = normalize(vec3(mx_world * vec4(local_tangent, 0.0)));
    vec3 world_bitangent = cross(world_normal, world_tangent) * a_Tangent.w;

    v_Tbn = mat3(world_tangent, world_bitangent, world_normal);
    v_Position = world_position.xyz / world_position.w;
    v_TexCoord = a_TexCoord;

    gl_Position = mx_mvp * mx_skin * local_position;
}
