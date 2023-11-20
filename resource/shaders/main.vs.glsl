
struct Vertex {
	vec2 pos;
	uint byte_color;
};

layout(binding=0) readonly buffer Vertices {
	Vertex s_vertices[];
};

layout(binding=0) uniform GlobalUniforms {
	mat4 u_projection;
};


out vec4 v_color;


void main() {
	Vertex vertex = s_vertices[gl_VertexID];

	gl_Position = u_projection * vec4(vertex.pos, 0.0, 1.0);
	v_color = unpackUnorm4x8(vertex.byte_color);
}