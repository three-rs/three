#define MAX_MORPH_TARGETS 9U

struct MorphTargetEntry {
    float weight;
    vec3 position_displacement;
    vec3 normal_displacement;
    vec3 tangent_displacement;
};

layout(std140) uniform b_MorphTargets {
    MorphTargetEntry u_MorphTargets[MAX_MORPH_TARGETS];
};
