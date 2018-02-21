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
uniform samplerBuffer b_Displacements;

//TODO: store each join transform in 3 vectors, similar to `i_WorldX`

mat4 fetch_joint_transform(int i) {
    vec4 col0 = texelFetch(b_JointTransforms, 4 * i + 0);
    vec4 col1 = texelFetch(b_JointTransforms, 4 * i + 1);
    vec4 col2 = texelFetch(b_JointTransforms, 4 * i + 2);
    vec4 col3 = texelFetch(b_JointTransforms, 4 * i + 3);

    return mat4(col0, col1, col2, col3);
}

vec3 fetch_displacement(uint i) {
    int index = gl_VertexID * int(MAX_TARGETS) + int(i);
    vec4 texel = texelFetch(b_Displacements, index);

    return texel.xyz;
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
    vec4 local_position = a_Position;
    vec4 local_normal = a_Normal;
    vec4 local_tangent = vec4(a_Tangent.xyz, 0.0);

    if (available(DISPLACEMENT_BUFFER)) {
        //TODO: store displacements in a 2D image of size (num_vertices, num_displacements).
        // That will allow a dynamic check for the number of actually stored displacements,
        // limiting the loop iteration count for trivial cases.
        for (uint i = 0U; i < MAX_TARGETS; ++i) {
            float disp = u_DisplacementContributions[i].weight * fetch_displacement(i);
            local_position.xyz += u_DisplacementContributions[i].position * disp;
            local_normal.xyz += u_DisplacementContributions[i].normal * disp;
            local_tangent.xyz += u_DisplacementContributions[i].tangent * disp;
        }
    }

    mat4 mx_world = transpose(mat4(i_World0, i_World1, i_World2, vec4(0.0, 0.0, 0.0, 1.0)));
    mat4 mx_mvp = u_ViewProj * mx_world;
    mat4 mx_skin = compute_skin_transform();

    vec4 world_position = mx_world * local_position;
    vec3 world_normal = normalize(vec3(mx_world * local_normal));
    vec3 world_tangent = normalize(vec3(mx_world * local_tangent));
    vec3 world_bitangent = cross(world_normal, world_tangent) * a_Tangent.w;

    v_Tbn = mat3(world_tangent, world_bitangent, world_normal);
    v_Position = world_position.xyz / world_position.w;
    v_TexCoord = a_TexCoord;

    gl_Position = mx_mvp * mx_skin * local_position;
}
