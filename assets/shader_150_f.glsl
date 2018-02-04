#version 150 core

uniform World {
    mat4 u_View;
    vec3 u_LightDir;
};

in vec3 v_Color;
in vec3 v_Normal;
out vec4 FragColor;

void main() {
  float diff = 0.005 + max(0.0, dot(normalize(v_Normal), u_LightDir))*0.995;
  FragColor = vec4(diff, diff, diff, 1.0);
  //FragColor = vec4(v_Normal, 1.0);
}