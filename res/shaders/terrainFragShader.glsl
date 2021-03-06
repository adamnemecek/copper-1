#version 400 core

const int NUM_LIGHTS = 4;

in vec2 pass_tex_coord;
in vec3 surface_normal;
in vec3 light_direction[NUM_LIGHTS];
in vec3 to_camera_dir;
// fog stuff
in float visibility;
in vec4 shadow_coords;

// rgba
layout(location = 0) out vec4 out_Color;
layout(location = 1) out vec4 out_brightness_Color;

uniform sampler2D background_sampler;
uniform sampler2D r_sampler;
uniform sampler2D g_sampler;
uniform sampler2D b_sampler;
uniform sampler2D blend_map_sampler;
uniform sampler2D shadow_map;

uniform vec3 light_color[NUM_LIGHTS];

uniform float shine_damper;
uniform float reflectivity;
// fog
uniform vec3 sky_color;
// point light attenuation
uniform vec3 attenuation[NUM_LIGHTS];

const bool uses_cell_shading = false;
const float brightness_levels = 2.0;

// how many pixels to sample on each side of center pixel (so 2 means 3x3 box) 
const int pcf_count = 1;
// texture pixels we will be sampling
const float texel_count = (pcf_count*2.0 + 1.0)*(pcf_count*2.0 + 1.0);
uniform float shadow_map_size;

void adjust_brightness(inout float diffuse_brightness, inout float specular_brightness) {
    if (!uses_cell_shading) {
        return;
    } else {
        // this assumes that the brightness is in [0,1] interval and so we use it to interpolate
        diffuse_brightness = floor(diffuse_brightness * brightness_levels) / brightness_levels;
        specular_brightness = floor(specular_brightness * brightness_levels) / brightness_levels;                
    }
}

void main(void) {
    // size of a pixel in texture coords space
    float texel_size = 1.0 / shadow_map_size;
    float total_in_shadow = 0.0;

    for (int x=-pcf_count; x<=pcf_count; x++) {
        for (int y=-pcf_count; y <= pcf_count; y++) {
            // compare depth with shadowmap depth to figure out if this piece of terrain is in shadow or not (absence of light due to something blocking it)
            float obj_depth_nearest_light = texture(shadow_map, shadow_coords.xy + vec2(x, y) * texel_size).r;
            // add slight offset to prevent shadow acne
            total_in_shadow += step(obj_depth_nearest_light + 0.005, shadow_coords.z);
        }
    }
    total_in_shadow /= texel_count;     
    float light_factor = 1.0 - total_in_shadow*shadow_coords.w;

    // sample untiled (by untiled i mean before coordinates are scaled by 40.0 which exploits REPEAT to tile the texture onto the object)
    vec4 blend_map_col = texture(blend_map_sampler, pass_tex_coord);
     // our bland map pixels are only either some val of r or g or b so this will be <= 1
    float background_coef = 1.0 - (blend_map_col.r + blend_map_col.g + blend_map_col.b);

    // tile the texture by scaling by 40.0 effectively getting 40 * 40 tiles of the same texture onto the model due to REPEAT rather than 1 tile)
    // this is useful if our textures are small and low detail but the object is large
    vec2 tiled_coords = pass_tex_coord * 40.0;
    vec4 background_col = texture(background_sampler, tiled_coords) * background_coef;
    vec4 r_col = texture(r_sampler, tiled_coords) * blend_map_col.r;
    vec4 g_col = texture(g_sampler, tiled_coords) * blend_map_col.g;
    vec4 b_col = texture(b_sampler, tiled_coords) * blend_map_col.b;
    vec4 blended_texture_color = background_col + r_col + g_col + b_col;

    // we have to normalize after interpolation
    vec3 unit_normal = normalize(surface_normal);
    vec3 unit_camera = normalize(to_camera_dir);

    vec3 total_diffuse = vec3(0.0);
    vec3 total_specular = vec3(0.0);
    for (int i=0; i<NUM_LIGHTS; i++) {
        float dist = length(light_direction[i]);
        float attenuation_factor = attenuation[i].x + attenuation[i].y * dist + attenuation[i].z * dist * dist;

        vec3 unit_light = normalize(light_direction[i]);    
        float dotNormToLight = dot(unit_normal, unit_light);
        float brightness = max(dotNormToLight, 0.0);
                        
        vec3 specular_reflection_dir = reflect(-light_direction[i], unit_normal);
        vec3 unit_specular_reflection = normalize(specular_reflection_dir);
        float dotSpecToCamera = dot(unit_camera, unit_specular_reflection);
        float spec_brightness = max(dotSpecToCamera, 0.0);

        adjust_brightness(brightness, spec_brightness);

        total_diffuse += (brightness * light_color[i]) / attenuation_factor; // add alpha of 1
        total_specular += (pow(spec_brightness, shine_damper) * reflectivity * light_color[i]) / attenuation_factor;
    }
    total_diffuse = max(total_diffuse * light_factor, 0.2); // clamp to [0.2, 1], the 0.2 means everything is given a little bit of color -> ambient
    
    vec4 light_based_out_color = vec4(total_diffuse, 1.0) * blended_texture_color + vec4(total_specular, 1.0);
    out_Color = mix(vec4(sky_color, 1.0), light_based_out_color, visibility);

    out_brightness_Color = vec4(0.0);
}