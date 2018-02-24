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
uniform sampler2D u_Displacements;

//TODO: store each join transform in 3 vectors, similar to `i_WorldX`

mat4 fetch_joint_transform(int i) {
    //Note: has to match `render::VECS_PER_BONE`
    vec4 row0 = texelFetch(b_JointTransforms, 3 * i + 0);
    vec4 row1 = texelFetch(b_JointTransforms, 3 * i + 1);
    vec4 row2 = texelFetch(b_JointTransforms, 3 * i + 2);

    return transpose(mat4(row0, row1, row2, vec4(0.0, 0.0, 0.0, 1.0)));
}

mat4 compute_skin_transform() {
    return
        a_JointWeights.x * fetch_joint_transform(a_JointIndices.x) +
        a_JointWeights.y * fetch_joint_transform(a_JointIndices.y) +
        a_JointWeights.z * fetch_joint_transform(a_JointIndices.z) +
        a_JointWeights.w * fetch_joint_transform(a_JointIndices.w);
}

bool available(int flag) {
    return (u_PbrFlags & flag) == flag;
}

void main() {
    vec3 local_position = a_Position.xyz;
    vec3 local_normal = a_Normal.xyz;
    vec3 local_tangent = a_Tangent.xyz;

    if (available(DISPLACEMENT_BUFFER)) {
        uint num_targets = uvec2(textureSize(u_Displacements, 0)).y / 3U;
        for (uint i = 0U; i < min(num_targets, MAX_TARGETS); ++i) {
            DisplacementContribution disp = u_DisplacementContributions[i];
            if (disp.weight == 0.0) continue;
            local_position += disp.position * disp.weight * texelFetch(u_Displacements, ivec2(gl_VertexID, 3U*i+0U), 0).xyz;
            local_normal   += disp.normal   * disp.weight * texelFetch(u_Displacements, ivec2(gl_VertexID, 3U*i+1U), 0).xyz;
            local_tangent  += disp.tangent  * disp.weight * texelFetch(u_Displacements, ivec2(gl_VertexID, 3U*i+2U), 0).xyz;
        }
    }

    mat4 mx_world = transpose(mat4(i_World0, i_World1, i_World2, vec4(0.0, 0.0, 0.0, 1.0)));
    mat4 mx_mvp = u_ViewProj * mx_world;
    mat4 mx_skin = compute_skin_transform();

    vec4 world_position = mx_world * vec4(local_position, a_Position.w);
    vec3 world_normal = mat3(mx_world) * normalize(local_normal);
    vec3 world_tangent = mat3(mx_world) * normalize(local_tangent);
    vec3 world_bitangent = cross(world_normal, world_tangent) * a_Tangent.w;

    v_Tbn = mat3(world_tangent, world_bitangent, world_normal);
    v_Position = world_position.xyz / world_position.w;
    v_TexCoord = a_TexCoord;

    gl_Position = mx_mvp * mx_skin * vec4(local_position, a_Position.w);
}
