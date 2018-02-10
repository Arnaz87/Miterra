#version 150 core

uniform World {
    mat4 u_View;
    vec3 u_LightDir;
};

uniform sampler2D t_Grass;
uniform sampler2D t_SoilSand;

in VertexData {
    vec3 normal;
    vec3 pos;
    flat int[3] materials;
    float[3] weights;
} VertexIn;

out vec4 FragColor;

vec3 sampleMaterial(vec2 uv, int i) {
  if (VertexIn.materials[i] == 0) {
    return texture(t_Grass, uv).rgb;
  } else {
    return texture(t_SoilSand, uv).rgb;
  }
}

void main() {

  vec2 uv = VertexIn.pos.xz;
  vec3 sample0 = sampleMaterial(uv, 0);
  vec3 sample1 = sampleMaterial(uv, 1);
  vec3 sample2 = sampleMaterial(uv, 2);

  vec3 sample = sample0 * VertexIn.weights[0]
              + sample1 * VertexIn.weights[1]
              + sample2 * VertexIn.weights[2];

  float diff = 0.005 + max(0.0, dot(normalize(VertexIn.normal), u_LightDir))*0.995;

  FragColor = vec4(diff * sample, 1.0);
}