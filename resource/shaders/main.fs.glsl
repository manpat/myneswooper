
in vec4 v_color;
in vec2 v_uv;

layout(binding=0) uniform sampler2D u_texture;

out vec4 o_color;

void main() {
	vec4 tex_color = texture(u_texture, v_uv);
	o_color = v_color * tex_color;
}