#version 150 core

uniform World {
    mat4 u_View;
    vec3 u_LightDir;
};

in vec3 a_Pos;
in vec3 a_Normal;
in int a_Material;

out VertexData {
    vec3 normal;
    vec3 pos;
    int material;
} VertexOut;

void main() {
    gl_Position = u_View * vec4(a_Pos, 1.0);
    VertexOut.normal = a_Normal;
    VertexOut.pos = a_Pos;
    VertexOut.material = a_Material;
}