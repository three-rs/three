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
#include <locals>
#include <globals>

in vec4 a_Position;
in vec2 a_TexCoord;
in vec4 a_Normal;
in vec4 a_Tangent;
in vec4 a_JointIndices;
in vec4 a_JointWeights;

in vec4 a_Displacement0;
in vec4 a_Displacement1;
in vec4 a_Displacement2;
in vec4 a_Displacement3;
in vec4 a_Displacement4;
in vec4 a_Displacement5;
in vec4 a_Displacement6;
in vec4 a_Displacement7;

out vec3 v_Position;
out vec2 v_TexCoord;
out mat3 v_Tbn;
out vec3 v_Normal;

struct DisplacementWeights {
    // weights.x => POSITION
    // weights.y => NORMAL
    // weights.z => TANGENT
    // weights.w => 0.0
    vec4 weights;
};

layout(std140) uniform b_DisplacementWeights {
    DisplacementWeights u_DisplacementWeights[8];
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

    position.xyz += u_DisplacementWeights[0].weights.x * a_Displacement0.xyz;
    position.xyz += u_DisplacementWeights[1].weights.x * a_Displacement1.xyz;
    position.xyz += u_DisplacementWeights[2].weights.x * a_Displacement2.xyz;
    position.xyz += u_DisplacementWeights[3].weights.x * a_Displacement3.xyz;
    position.xyz += u_DisplacementWeights[4].weights.x * a_Displacement4.xyz;
    position.xyz += u_DisplacementWeights[5].weights.x * a_Displacement5.xyz;
    position.xyz += u_DisplacementWeights[6].weights.x * a_Displacement6.xyz;
    position.xyz += u_DisplacementWeights[7].weights.x * a_Displacement7.xyz;

    return position;
}

vec3 compute_world_normal()
{
    vec3 normal = a_Normal.xyz;

    normal.xyz += u_DisplacementWeights[0].weights.y * a_Displacement0.xyz;
    normal.xyz += u_DisplacementWeights[1].weights.y * a_Displacement1.xyz;
    normal.xyz += u_DisplacementWeights[2].weights.y * a_Displacement2.xyz;
    normal.xyz += u_DisplacementWeights[3].weights.y * a_Displacement3.xyz;
    normal.xyz += u_DisplacementWeights[4].weights.y * a_Displacement4.xyz;
    normal.xyz += u_DisplacementWeights[5].weights.y * a_Displacement5.xyz;
    normal.xyz += u_DisplacementWeights[6].weights.y * a_Displacement6.xyz;
    normal.xyz += u_DisplacementWeights[7].weights.y * a_Displacement7.xyz;

    return normalize(vec3(u_World * vec4(normal, 0.0)));
}

vec3 compute_world_tangent()
{
    vec3 tangent = a_Tangent.xyz;

    tangent.xyz += u_DisplacementWeights[0].weights.z * a_Displacement0.xyz;
    tangent.xyz += u_DisplacementWeights[1].weights.z * a_Displacement1.xyz;
    tangent.xyz += u_DisplacementWeights[2].weights.z * a_Displacement2.xyz;
    tangent.xyz += u_DisplacementWeights[3].weights.z * a_Displacement3.xyz;
    tangent.xyz += u_DisplacementWeights[4].weights.z * a_Displacement4.xyz;
    tangent.xyz += u_DisplacementWeights[5].weights.z * a_Displacement5.xyz;
    tangent.xyz += u_DisplacementWeights[6].weights.z * a_Displacement6.xyz;
    tangent.xyz += u_DisplacementWeights[7].weights.z * a_Displacement7.xyz;

    return normalize(vec3(u_World * vec4(tangent, 0.0)));
}

void main()
{
    mat4 mx_mvp = u_ViewProj * u_World;
    mat4 mx_skin = compute_skin_transform();

    vec4 local_position = compute_local_position();
    vec4 world_position = u_World * local_position;
    vec3 world_normal = compute_world_normal();
    vec3 world_tangent = compute_world_tangent();
    vec3 world_bitangent = cross(world_normal, world_tangent) * a_Tangent.w;

    v_Tbn = mat3(world_tangent, world_bitangent, world_normal);
    v_Position = world_position.xyz / world_position.w;
    v_TexCoord = a_TexCoord;

    gl_Position = mx_mvp * mx_skin * local_position;
}
