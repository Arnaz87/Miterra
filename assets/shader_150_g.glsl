#version 150 core

layout(triangles) in;
layout(triangle_strip, max_vertices=3) out;

uniform World {
    mat4 u_View;
    vec3 u_LightDir;
};

in VertexData {
    vec3 normal;
    vec3 pos;
    int material;
} VertexIn[3];

out VertexData {
    vec3 normal;
    vec3 pos;
    flat int[3] materials;
    float[3] weights;
} VertexOut;

void copyVertex (int i) {
    gl_Position = gl_in[i].gl_Position;
    VertexOut.normal = VertexIn[i].normal;
    VertexOut.pos = VertexIn[i].pos;
}

void setWeight (int i) {
    VertexOut.weights = float[3](0.0, 0.0, 0.0);
    VertexOut.weights[i] = 1.0;
}

void main() {
    // The same material array is shared by all the vertices.
    VertexOut.materials = int[3](
        VertexIn[0].material,
        VertexIn[1].material,
        VertexIn[2].material
    );

    copyVertex(0);
    setWeight(0);
    EmitVertex();

    copyVertex(1);
    setWeight(1);
    EmitVertex();

    copyVertex(2);
    setWeight(2);
    EmitVertex();
}