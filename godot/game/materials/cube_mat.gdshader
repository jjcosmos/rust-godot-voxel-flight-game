shader_type spatial;
uniform sampler2D gradient_sampler_x;
uniform sampler2D gradient_sampler_y;
uniform sampler2D gradient_sampler_z;

uniform float distance_scale;

void vertex() {
	// Called for every vertex the material is visible on.
}

void fragment() {
	vec3 world = (INV_VIEW_MATRIX * vec4(VERTEX, 1.0)).xyz * distance_scale;
	world += vec3(TIME, TIME, TIME) * 0.05;
	vec3 x = texture(gradient_sampler_x, vec2(world.x * 1.1, 0)).rgb;
	vec3 y = texture(gradient_sampler_y, vec2(world.y * 0.5, 0)).rgb;
	vec3 z = texture(gradient_sampler_z, vec2(world.z *  1.5, 0)).rgb;
	
	ALBEDO = mix(mix(x,y,0.5),z, 0.25);
	//ALBEDO = vec3(1.,0.,0.);
	SPECULAR = 0.5;
	ROUGHNESS = 1.0;
	// Called for every pixel the material is visible on.
}

//void light() {
	// Called for every pixel for every light affecting the material.
	// Uncomment to replace the default light processing function with this one.
//}
