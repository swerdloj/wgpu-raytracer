/*
   Interfaces don't work in glslang's hlsl to spirv
*/

#define dot2(x) dot(x, x)

// Color accumulator for multi-sample averaging
layout(set = 0, binding = 0) RWTexture2D<float4> storage_image;

// Raytracer parameters/inputs
layout(set = 1, binding = 0)
cbuffer Uniforms {
    // Explicit offsets for debugging
    /* layout(offset = 0)  */ float2 window_size;     // Window dimensions
    /* layout(offset = 8)  */ uint sample_number;     // The current sample number (starting at 1)
    /* layout(offset = 12) */ uint samples_per_pixel; // Rays fired per pixel
    /* layout(offset = 16) */ uint max_ray_bounces;   // Max bounces per ray (path length)
    /* layout(offset = 20) */ float v_fov;            // Vertical field of view

    /* layout(offset = 32) */ float3 camera_position; // Camera location (look from)
    /* layout(offset = 48) */ float3 camera_lookat;   // Camera lookat position
};
// This is because my image is still upside down...
static float3 camera_lookat2 = camera_lookat * float3(1, -1, 1);


// TODO: For fake inerfaces, inherit from a base class which has:
//       One member variable to identify which super-type (like tagged union)
//       The pre-implemented methods would then call a function defined elsewhere
//       using a switch-case on the super-type.
//       This would allow for the likes of storing different shapes in the same array
//       TODO: Try implementing this idea for Material and see how it works.

// TODO: To fix the Material/HitRecord issue, try defining each in their own header, then have each import the other's header.


const float PI = 3.141592;
const float FAR_PLANE_DIST = 10000.0;


/********** Random Number Generation **********/

// Hashes translated from Dave_Hoskins shader: https://www.shadertoy.com/view/4djSRW
float hash11(float p) {
    p = frac(p * 0.1031);
    p *= p + 33.33;
    p *= p + p;
    return frac(p);
}
float hash12(float2 p) {
    float3 p3 = frac(float3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return frac((p3.x + p3.y) * p3.z);
}

static float2 rand_state;
// Random float on [0, 1)
float random() {
    rand_state.x = hash12(rand_state * 79.1233);
    rand_state.y = hash12(rand_state * 173.9);

    return rand_state.x;
}
// Random float on [min, max)
float rand_range(float _min, float _max) {
    return _min + (_max - _min) * random();
}

// Translated from https://www.shadertoy.com/view/lssBD7
float3 random_in_unit_sphere() {
    float phi = 2 * PI * random();
    float cos_theta = 2 * random() - 1;
    float u = random();

    float theta = acos(cos_theta);
    float r = pow(u, 1./3.);

    float x = r * sin(theta) * cos(phi);
    float y = r * sin(theta) * sin(phi);
    float z = r * cos_theta;

    return float3(x, y, z);
}

float3 random_unit_vector() {
    float a = rand_range(0, 2*PI);
    float z = rand_range(-1, 1);
    float r = sqrt(1 - z*z);
    return float3(r*cos(a), r*sin(a), z);
}

float3 random_in_hemisphere(float3 normal) {
    float3 in_unit_sphere = random_in_unit_sphere();

    if (dot(in_unit_sphere, normal) > 0.0) {
        return in_unit_sphere;
    } else {
        return -in_unit_sphere;
    }
}

// From https://www.shadertoy.com/view/MtycDD
float2 random_in_unit_disk() {
    random();
    float2 h = rand_state * float2(1, 2*PI);
    float phi = h.y;
    float r = sqrt(h.x);
    return r * float2(sin(phi), cos(phi));
}

/********** Ray **********/
// Classes & interfaces in HLSL put GLSL to shame
class Ray {
    float3 origin;
    float3 direction;

    float3 position(float t) {
        return origin + t*direction;
    }
};


/********** Materials **********/

#define MAT_METAL 1
#define MAT_LAMBERTIAN 2
#define MAT_DIELECTRIC 3

struct Material {
    uint type;

    float3 albedo;
    float metalic_fuzz;
    float dielectric_index_of_refraction;
};

float schlick_approx(float cosine, float index_of_refraction) {
    float r0 = (1 - index_of_refraction) / (1 + index_of_refraction);
    r0 = r0 * r0;
    return r0 + (1 - r0) * pow((1 - cosine), 5);
}



/********** Interfaces **********/

struct HitRecord {
    float3 position;
    float3 normal;
    float distance;
    bool is_front_face;
    Material material;

    void set_face_normal(Ray ray, float3 outward_normal) {
        is_front_face = dot(ray.direction, outward_normal) < 0;
        normal = is_front_face ? outward_normal : -outward_normal;
    }
};

// Static methods don't work either.....
namespace Material_ {
    Material create_metal(float3 albedo, float metalic_fuzz) {
        Material mat = {MAT_METAL, albedo, metalic_fuzz, 0}; 
        return mat;
    }

    Material create_lambertian(float3 albedo) {
        Material mat = {MAT_LAMBERTIAN, albedo, 0, 0}; 
        return mat;
    }

    Material create_dielectric(float index_of_refraction) {
        Material mat = {MAT_DIELECTRIC, 0, 0, index_of_refraction};
        return mat;
    }

    // FIXME: I can't put this inside Material because of circular dependency, and 
    // there is no struct/class forward declaration in HLSL.....
    bool scatter_ray(Material material, Ray ray_in, HitRecord record, out float3 attenuation, out Ray scattered_ray) {
        switch (material.type) {
            // Matte
            case MAT_LAMBERTIAN: {
                float3 scatter_direction = record.normal + random_unit_vector();
                scattered_ray.origin = record.position;
                scattered_ray.direction = scatter_direction;
                
                attenuation = material.albedo;
                return true;
            }
            // Metal
            case MAT_METAL: {
                float3 reflected = reflect(normalize(ray_in.direction), record.normal);
                scattered_ray.origin = record.position;
                scattered_ray.direction = reflected + material.metalic_fuzz*random_in_unit_sphere();
                
                attenuation = material.albedo;
                return dot(scattered_ray.direction, record.normal) > 0;
            }
            // Glass
            case MAT_DIELECTRIC: {
                attenuation = float3(1);
                
                float etai_over_etat = (record.is_front_face) ? (1/material.dielectric_index_of_refraction) : material.dielectric_index_of_refraction;

                float3 unit_direction = normalize(ray_in.direction);

                float cos_theta = min(dot(-unit_direction, record.normal), 1);
                float sin_theta = sqrt(1 - cos_theta*cos_theta);

                if (etai_over_etat * sin_theta > 1) {
                    float3 reflected = reflect(unit_direction, record.normal);
                    scattered_ray.origin = record.position;
                    scattered_ray.direction = reflected;
                    return true;
                }
                
                float reflect_chance = schlick_approx(cos_theta, etai_over_etat);
                if (random() < reflect_chance) {
                    float3 reflected = reflect(unit_direction, record.normal);
                    scattered_ray.origin = record.position;
                    scattered_ray.direction = reflected;
                    return true;
                }

                float3 refracted = refract(unit_direction, record.normal, etai_over_etat);
                scattered_ray.origin = record.position;
                scattered_ray.direction = refracted;
                return true;
            }

            // Unreachable
            default: return false;
        }
    }
};



// FIXME: Interfaces don't work with spirv.....
// interface IHittable {
//     bool intersect(Ray ray, float dist_min, float dist_max, out HitRecord record);
// };


/********** Shapes **********/

class Sphere {
    float3 center;
    float radius;
    Material material;

    // Check sphere hit using quadratic formula
    bool intersect(Ray ray, float dist_min, float dist_max, out HitRecord record) {
        float3 direction = ray.origin - center;

        float a = dot2(ray.direction);
        float half_b = dot(direction, ray.direction);
        float c = dot2(direction) - radius * radius;
        float discriminant = half_b * half_b - a * c;

        if (discriminant > 0) {
            float root = sqrt(discriminant);
            float distance = (-half_b - root) / a;

            if (distance < dist_max && distance > dist_min) {
                record.distance = distance;
                record.position = ray.position(distance);
                float3 outward_normal = (record.position - center) / radius;
                record.set_face_normal(ray, outward_normal);

                record.material = material;

                return true;
            }

            distance = (-half_b + root) / a;
            if (distance < dist_max && distance > dist_min) {
                record.distance = distance;
                record.position = ray.position(distance);
                float3 outward_normal = (record.position - center) / radius;
                record.set_face_normal(ray, outward_normal);

                record.material = material;

                return true;
            }
        }

        return false;
    } // intersect()
};


/********** Camera **********/

class Camera {
    float3 position;
    float3 bottom_left;
    float3 horizontal;
    float3 vertical;
    float v_fov;
    float3 u, v, w;
    float lens_radius;

    Ray create_ray(float2 uv) {
        float2 direction = lens_radius * random_in_unit_disk();
        float3 offset = u * direction.x + v * direction.y;

        Ray ray = { position + offset, 
                    bottom_left + uv.x*horizontal + uv.y*vertical - position - offset
        };
        return ray;
    }
};

namespace Camera_ {
    Camera create(float3 position, float3 lookat, float3 v_up, float v_fov, float aperature, float focal_dist) {
        float theta = radians(v_fov);

        float viewport_height = 2 * tan(theta/2);
        float viewport_width = viewport_height * (window_size.x / window_size.y);
        
        float3 w = normalize(position - lookat);
        float3 u = normalize(cross(v_up, w));
        float3 v = cross(w, u);

        float3 horizontal = viewport_width * u;
        float3 vertical = viewport_height * v;
        float3 bottom_left = position - horizontal/2 - vertical/2 - focal_dist * w;

        Camera camera = {position, bottom_left, horizontal, vertical, v_fov, u, v, w, aperature / 2};
        return camera;
    }
}


/********** Main **********/

bool scene(Ray ray, float dist_min, float dist_max, inout HitRecord record) {
    Sphere small_sphere = { float3(0, 0, -1), 0.5, 
        Material_::create_lambertian(float3(0.1, 0.2, 0.5)),
    };
    Sphere big_sphere = { float3(0, -100.5, -1), 100,
        Material_::create_lambertian(float3(0.8, 0.8, 0.0)),
    };
    Sphere glass1 = { float3(-1.05, 0, -1), 0.5, 
        Material_::create_dielectric(1.5),
    };
    Sphere glass2 = { float3(-1.05, 0, -1), -0.45, 
        Material_::create_dielectric(1.5),
    };
    Sphere metal1 = { {1.05, 0, -1}, 0.5, 
        Material_::create_metal(float3(0.8, 0.6, 0.2), 0.1),
    };
    Sphere metal2 = { {-1, 0, -1}, 0.5, 
        Material_::create_metal(float3(0.8, 0.8, 0.8), 0.3),
    };

    // Sphere lookat_pos = { camera_position + camera_lookat2, 0.02,
    //     Material_::create_metal(float3(0.4, 0, 0), 0.2),
    // };
    
    const uint num_spheres = 5;
    Sphere spheres[] = {glass1, glass2, big_sphere, metal1, small_sphere};

    bool hit_anything = false;
    float closest_hit = dist_max;

    HitRecord temp_record;

    for (uint i = 0; i < num_spheres; ++i) {
        if ( spheres[i].intersect(ray, dist_min, closest_hit, temp_record) ) {
            hit_anything = true;
            closest_hit = temp_record.distance;
            record = temp_record;
        }
    }

    return hit_anything;
}

float3 sky_color(Ray ray) {
    float3 unit_direction = normalize(ray.direction);
    float t = 0.5 * (unit_direction.y + 1.);
    return (1 - t) * float3(1) + t * float3(0.5, 0.7, 1.0);
}

float3 fire_ray(Ray ray) {
    HitRecord record;
    Ray scattered_ray;

    float3 attenuation;
    float3 color = 1;

    for (uint depth = 0; depth < max_ray_bounces; ++depth) {
        // If hit scene
        if ( scene(ray, 0.001, FAR_PLANE_DIST, record) ) {
            // If ray scattered
            if ( Material_::scatter_ray(record.material, ray, record, attenuation, scattered_ray) ) {
                ray = scattered_ray;
                color *= attenuation;
            } else {
                color *= 0;
                break;
            }
        } else {
            color *= sky_color(ray);
            break;
        }
    }

    return color;
}


float4 main(float4 pixel_coords : SV_POSITION) : COLOR0 {
    rand_state = (pixel_coords.xy / window_size) + sample_number * 15.23;

    // TODO: The camera needs to be redone so these can be removed
    // TODO: Calculate focus distance by querying the distance to scene from camera
    float3 cam_position = {0, 0, 5};
    float3 lookat = {0, 0, -1};
    float3 v_up = {0, 1, 0};
    float focal_dist = length(cam_position - lookat);

    Camera camera = Camera_::create(
        camera_position,// Position
        camera_lookat2 + camera_position,         // Lookat
        v_up,           // Up vector
        v_fov,          // Vertical field of view
        0.0,            // Aperature size
        focal_dist      // Focal plane dist
    );

    float3 color = 0;
    
    for (uint i = 0; i < samples_per_pixel; ++i) {
        float2 uv = (pixel_coords.xy + float2(random(), random())) / window_size;
        // uv is still flipped.......
        uv.y = 1 - uv.y;

        color += fire_ray(camera.create_ray( uv ));
    }
    color /= samples_per_pixel;

    // Gamma correction (gamma = 2.2)
    color = pow(color, 0.4545);
    // Clamp color on [0, 1)
    color = smoothstep(0, 1, saturate(color));


    uint2 image_coords = uint2(pixel_coords.xy);
    if (sample_number > 1) {
        color += storage_image[image_coords].rgb;
    }
    storage_image[image_coords] = float4(color, sample_number);

    return float4(color / sample_number, 1);
}