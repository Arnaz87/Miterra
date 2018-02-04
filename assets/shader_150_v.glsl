#version 150 core

uniform World {
    mat4 u_View;
    vec3 u_LightDir;
};

in vec3 a_Pos;
in vec3 a_Color;
in vec3 a_Normal;
out vec3 v_Color;
out vec3 v_Normal;

void main() {
    v_Color = a_Color;
    v_Normal = a_Normal;
    gl_Position = u_View * vec4(a_Pos, 1.0);
}