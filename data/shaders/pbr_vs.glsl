
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

in vec4 a_Position;
in vec4 a_Normal;
in vec4 a_Tangent;
in vec2 a_TexCoord;

uniform b_PerVertexParams {
    mat4 u_Mvp;
    mat4 u_Model;
};

out vec3 v_Position;
out vec2 v_TexCoord;
out mat3 v_Tbn;
out vec3 v_Normal;

void main()
{
    vec4 pos = u_Model * a_Position;
    v_Position = vec3(pos.xyz) / pos.w;

    vec3 normal = normalize(vec3(u_Model * vec4(a_Normal.xyz, 0.0)));
    vec3 tangent = normalize(vec3(u_Model * vec4(a_Tangent.xyz, 0.0)));
    vec3 bitangent = cross(normal, tangent) * a_Tangent.w;
    v_Tbn = mat3(tangent, bitangent, normal);

    v_TexCoord = a_TexCoord;

    gl_Position = u_Mvp * a_Position;
}


