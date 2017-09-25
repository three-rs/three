#define MAX_LIGHTS  4U

struct Light {
    mat4 projection;
    vec4 pos;
    vec4 dir;
    vec4 focus;
    vec4 color;
    vec4 color_back;
    vec4 intensity;
    ivec4 shadow_params;
};

layout(std140) uniform b_Lights {
    Light u_Lights[MAX_LIGHTS];
};
