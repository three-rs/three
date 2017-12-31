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
#include <globals>
#include <morph_targets>

in vec4 a_Position;
in vec2 a_TexCoord;
in vec4 a_Normal;
in vec4 a_Tangent;
in vec4 a_JointIndices;
in vec4 a_JointWeights;

out vec3 v_Position;
out vec2 v_TexCoord;
out mat3 v_Tbn;
out vec3 v_Normal;

in vec4 i_World0;
in vec4 i_World1;
in vec4 i_World2;
in vec4 i_MatParams;
in vec4 i_Color;
in vec4 i_UvRange;

// Toggles displacement contributions to `a_Position/a_Normal/a_Tangent`.
struct DisplacementContribution {
    // position: 1.0 if morph target weights should influence a_Position
    // normal: 1.0 if morph target weights should influence a_Normal
    // tangent: 1.0 if morph target weights should influence a_Tangent
    // weight: The weight to be applied.
    float position, normal, tangent, weight;
};

layout(std140) uniform b_DisplacementContributions {
    DisplacementContribution u_DisplacementContributions[8];
};

uniform samplerBuffer b_JointTransforms;

mat4 fetch_joint_transform(int i)
{
    vec4 col0 = texelFetch(b_JointTransforms, 4 * i);
    vec4 col1 = texelFetch(b_JointTransforms, 4 * i + 1);
    vec4 col2 = texelFetch(b_JointTransforms, 4 * i + 2);
    vec4 col3 = texelFetch(b_JointTransforms, 4 * i + 3);

    return mat4(col0, col1, col2, col3);
}

mat4 compute_skin_transform()
{
    return
	a_JointWeights.x * fetch_joint_transform(int(a_JointIndices.x)) +
	a_JointWeights.y * fetch_joint_transform(int(a_JointIndices.y)) +
	a_JointWeights.z * fetch_joint_transform(int(a_JointIndices.z)) +
	a_JointWeights.w * fetch_joint_transform(int(a_JointIndices.w));
}

vec4 compute_local_position()
{
    vec4 position = a_Position;

    position.xyz += u_DisplacementContributions[0].position * u_DisplacementContributions[0].weight * a_Displacement0.xyz;
    position.xyz += u_DisplacementContributions[1].position * u_DisplacementContributions[1].weight * a_Displacement1.xyz;
    position.xyz += u_DisplacementContributions[2].position * u_DisplacementContributions[2].weight * a_Displacement2.xyz;
    position.xyz += u_DisplacementContributions[3].position * u_DisplacementContributions[3].weight * a_Displacement3.xyz;
    position.xyz += u_DisplacementContributions[4].position * u_DisplacementContributions[4].weight * a_Displacement4.xyz;
    position.xyz += u_DisplacementContributions[5].position * u_DisplacementContributions[5].weight * a_Displacement5.xyz;
    position.xyz += u_DisplacementContributions[6].position * u_DisplacementContributions[6].weight * a_Displacement6.xyz;
    position.xyz += u_DisplacementContributions[7].position * u_DisplacementContributions[7].weight * a_Displacement7.xyz;

    return position;
}

vec3 compute_world_normal()
{
    vec3 normal = a_Normal.xyz;

    normal += u_DisplacementContributions[0].normal * u_DisplacementContributions[0].weight * a_Displacement0.xyz;
    normal += u_DisplacementContributions[1].normal * u_DisplacementContributions[1].weight * a_Displacement1.xyz;
    normal += u_DisplacementContributions[2].normal * u_DisplacementContributions[2].weight * a_Displacement2.xyz;
    normal += u_DisplacementContributions[3].normal * u_DisplacementContributions[3].weight * a_Displacement3.xyz;
    normal += u_DisplacementContributions[4].normal * u_DisplacementContributions[4].weight * a_Displacement4.xyz;
    normal += u_DisplacementContributions[5].normal * u_DisplacementContributions[5].weight * a_Displacement5.xyz;
    normal += u_DisplacementContributions[6].normal * u_DisplacementContributions[6].weight * a_Displacement6.xyz;
    normal += u_DisplacementContributions[7].normal * u_DisplacementContributions[7].weight * a_Displacement7.xyz;

    return normalize(vec3(u_World * vec4(normal, 0.0)));
}

vec3 compute_world_tangent()
{
    vec3 tangent = a_Tangent.xyz;

    tangent += u_DisplacementContributions[0].tangent * u_DisplacementContributions[0].weight * a_Displacement0.xyz;
    tangent += u_DisplacementContributions[1].tangent * u_DisplacementContributions[1].weight * a_Displacement1.xyz;
    tangent += u_DisplacementContributions[2].tangent * u_DisplacementContributions[2].weight * a_Displacement2.xyz;
    tangent += u_DisplacementContributions[3].tangent * u_DisplacementContributions[3].weight * a_Displacement3.xyz;
    tangent += u_DisplacementContributions[4].tangent * u_DisplacementContributions[4].weight * a_Displacement4.xyz;
    tangent += u_DisplacementContributions[5].tangent * u_DisplacementContributions[5].weight * a_Displacement5.xyz;
    tangent += u_DisplacementContributions[6].tangent * u_DisplacementContributions[6].weight * a_Displacement6.xyz;
    tangent += u_DisplacementContributions[7].tangent * u_DisplacementContributions[7].weight * a_Displacement7.xyz;

    return normalize(vec3(u_World * vec4(tangent, 0.0)));
}

void main()
{
    mat4 mx_world = transpose(mat4(i_World0, i_World1, i_World2, vec4(0.0, 0.0, 0.0, 1.0)));
    mat4 mx_mvp = u_ViewProj * mx_world;
    mat4 mx_skin = compute_skin_transform();

    vec4 local_position = compute_local_position();
    vec4 world_position = mx_world * local_position;
    vec3 world_normal = compute_world_normal();
    vec3 world_tangent = compute_world_tangent();
    vec3 world_bitangent = cross(world_normal, world_tangent) * a_Tangent.w;

    v_Tbn = mat3(world_tangent, world_bitangent, world_normal);
    v_Position = world_position.xyz / world_position.w;
    v_TexCoord = a_TexCoord;

    gl_Position = mx_mvp * mx_skin * local_position;
}
