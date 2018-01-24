#version 150 core

uniform Transform {
    mat4 u_Transform;
};

in vec3 a_Pos;
in vec3 a_Color;
in vec3 a_Normal;
out vec3 v_Color;
out vec3 v_Normal;

void main() {
    v_Color = a_Color;
    v_Normal = a_Normal;
    gl_Position = u_Transform * vec4(a_Pos, 1.0);
}