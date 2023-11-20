
struct Vertex {
	vec2 pos;
	uint byte_color;
	uint uv_and_index;
};

layout(binding=0) readonly buffer Vertices {
	Vertex s_vertices[];
};

layout(binding=0) uniform GlobalUniforms {
	mat4 u_projection;
};

layout(binding=0) uniform sampler2D u_texture;

out vec4 v_color;
out vec2 v_uv;


void main() {
	Vertex vertex = s_vertices[gl_VertexID];

	vec2 base_uv = unpackUnorm4x8(vertex.uv_and_index).xy;
	uint texture_index = bitfieldExtract(vertex.uv_and_index, 16, 16);

	ivec2 image_size = textureSize(u_texture, 0);
	vec2 cell_size = vec2(128.0) / vec2(image_size);

	vec2 uv = base_uv * cell_size + vec2(cell_size.x * float(texture_index), 0.0);

	gl_Position = u_projection * vec4(vertex.pos, 0.0, 1.0);
	v_color = unpackUnorm4x8(vertex.byte_color);
	v_uv = uv;
}