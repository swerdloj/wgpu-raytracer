#version 450

// Output image format
layout(rgba32f, set = 0, binding = 0) uniform image2D img_output;

layout(set = 1, binding = 0)
uniform Uniforms {
    vec2 window_size;
    uint sample_number;
    uint samples_per_pixel;
    uint max_ray_bounces;
};


layout(location = 0) out vec4 out_color;


const float PI = 3.141592;
const float MAX_FLOAT = 99999.99;


// See Dave_Hoskins for more hashes: https://www.shadertoy.com/view/4djSRW
float hash11(float p) {
    p = fract(p * .1031);
    p *= p + 33.33;
    p *= p + p;
    return fract(p);
}
float hash12(vec2 p) {
    vec3 p3  = fract(vec3(p.xyx) * .1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// Random float on [0, 1)
vec2 rand_state;
float random() {
    rand_state.x = hash12(rand_state);
    rand_state.y = hash12(rand_state);

    return rand_state.x;
}

// Random float on [min, max)
float rand_range(float min, float max) {
    return min + (max - min) * random();
}

vec3 random_unit_vector() {
    float a = rand_range(0, 2.*PI);
    float z = rand_range(-1, 1);
    float r = sqrt(1 - z*z);

    return vec3(r*cos(a), r*sin(a), z);
}

// From https://www.shadertoy.com/view/lssBD7
vec3 random_in_unit_disk() {
    float spx = 2.0 * random() - 1.0;
    float spy = 2.0 * random() - 1.0;

    float r, phi;

    if(spx > -spy) {
        if(spx > spy) {
            r = spx;
            phi = spy / spx;
        } else {
            r = spy;
            phi = 2.0 - spx / spy;
        }
    } else {
        if(spx < spy) {
            r = -spx;
            phi = 4.0f + spy / spx;
        } else {
            r = -spy;
            if(spy != 0.0)
                phi = 6.0 - spx / spy;
            else
                phi = 0.0;
        }
    }
    phi *= PI / 4.0;

    return vec3(r * cos(phi), r * sin(phi), 0.0);
}

// From https://www.shadertoy.com/view/lssBD7
vec3 random_in_unit_sphere() {
    float phi = 2.0 * PI * random();
    float cosTheta = 2.0 * random() - 1.0;
    float u = random();

    float theta = acos(cosTheta);
    float r = pow(u, 1.0 / 3.0);

    float x = r * sin(theta) * cos(phi);
    float y = r * sin(theta) * sin(phi);
    float z = r * cos(theta);

    return vec3(x, y, z);
}

vec3 random_in_hemisphere(vec3 normal) {
    vec3 in_unit_sphere = random_in_unit_sphere();

    if (dot(in_unit_sphere, normal) > 0.0) {
        return in_unit_sphere;
    } else {
        return -in_unit_sphere;
    }
}

float schlick_approx(float cosine, float index_of_refraction) {
    float r0 = (1 - index_of_refraction) / (1 + index_of_refraction);
    r0 = r0 * r0;

    return r0 + (1 - r0)*pow((1 - cosine), 5);
}


// BEGIN RAY //
struct HitRecord {
    vec3 point;
    vec3 normal;
    float dist;
    bool front_face;

    uint material_type;
    vec3 albedo;
    float fuzziness;
    float index_of_refraction;
};

struct Ray {
    vec3 origin;
    vec3 direction;
};

vec3 Ray_position(Ray ray, float dist) {
    return ray.origin + dist * ray.direction;
}

void HitRecord_set_face_normal(inout HitRecord self, Ray ray, vec3 outward_normal) {
    self.front_face = dot(ray.direction, outward_normal) < 0;
    self.normal = self.front_face ? outward_normal : -outward_normal;
}
// END RAY //


// BEGIN CAMERA  //
struct Camera {
	vec3 position;
    vec3 bottom_left;
    vec3 horizontal;
    vec3 vertical;
    vec3 u, v, w;
    float lens_radius;
};

Camera Camera_new(vec3 position, vec3 lookat, vec3 v_up, float v_fov, float aspect_ratio, float aperature, float focus_dist) {
    float theta = radians(v_fov);
    float viewport_height = 2. * tan(theta / 2.);
    float viewport_width = aspect_ratio * viewport_height;

    vec3 w = normalize(position - lookat); // Forward
    vec3 u = normalize(cross(v_up, w)); // Right
    vec3 v = cross(w, u); // Up

    vec3 horizontal = viewport_width * u;
    vec3 vertical = viewport_height * v;
    vec3 bottom_left = position - horizontal/2. - vertical/2. - focus_dist * w;

    float lens_radius = aperature / 2.;

    return Camera(position, bottom_left, horizontal, vertical, u, v, w, lens_radius);
}

Ray Camera_get_ray(in Camera self, vec2 uv) {
    vec3 direction = self.lens_radius * random_in_unit_disk();
    vec3 offset = self.u * direction.x + self.v * direction.y;

    return Ray(self.position + offset, 
               self.bottom_left + uv.x*self.horizontal + uv.y*self.vertical - self.position - offset);
}
// END_CAMERA //


// BEGIN MATERIALS // 
#define LAMBERTIAN 1
#define METAL 2
#define DIELECTRIC 3

bool Lambertian_scatter(Ray ray_in, HitRecord record, out vec3 attenuation, out Ray scattered) {
    vec3 scatter_direction = record.normal + random_unit_vector();
    scattered = Ray(record.point, scatter_direction);
    attenuation = record.albedo;

    return true;
}

bool Metal_scatter(Ray ray_in, HitRecord record, out vec3 attenuation, out Ray scattered) {
    vec3 reflected = reflect(ray_in.direction, record.normal);
    scattered = Ray(record.point, reflected + record.fuzziness*random_in_unit_sphere());
    attenuation = record.albedo;
    return (dot(scattered.direction, record.normal) > 0);
}

bool Dielectric_scatter(Ray ray_in, HitRecord record, out vec3 attenuation, out Ray scattered) {
    // Glass absorbs nothing
    attenuation = vec3(1);

    float eta;
    if (record.front_face) {
        eta = 1. / record.index_of_refraction;
    } else {
        eta = record.index_of_refraction;
    }

    vec3 unit_direction = normalize(ray_in.direction);
    
    float cos_theta = min(dot(-unit_direction, record.normal), 1.0);
    float sin_theta = sqrt(1. - cos_theta*cos_theta);

    if (eta * sin_theta > 1.) {
        vec3 reflected = reflect(unit_direction, record.normal);
        scattered = Ray(record.point, reflected);
        return true;
    }

    float reflect_chance = schlick_approx(cos_theta, eta);
    if (random() < reflect_chance) {
        vec3 reflected = reflect(unit_direction, record.normal);
        scattered = Ray(record.point, reflected);
        return true;
    }

    vec3 refracted = refract(unit_direction, record.normal, eta);
    scattered = Ray(record.point, refracted);
    return true;
}

bool Material_scatter(Ray ray, HitRecord record, inout vec3 attenuation, inout Ray scattered) {
    if (record.material_type == LAMBERTIAN) {
        return Lambertian_scatter(ray, record, attenuation, scattered);
    }
    if (record.material_type == METAL) {
        return Metal_scatter(ray, record, attenuation, scattered);
    }
    if (record.material_type == DIELECTRIC) {
        return Dielectric_scatter(ray, record, attenuation, scattered);
    }

    // Unreachable
    return false;
}
// END MATERIALS //


// BEGIN SHAPES //
struct Sphere {
    vec3 center;
    float radius;

    uint material_type;
    vec3 albedo;
    float fuzziness;
    float index_of_refraction;
};

// Sphere collision using quadratic formula
bool Sphere_hit(Sphere sphere, Ray ray, float dist_min, float dist_max, inout HitRecord record) {
    vec3 direction = ray.origin - sphere.center;

    float a = dot(ray.direction, ray.direction);
    float half_b = dot(direction, ray.direction);
    float c = dot(direction, direction) - sphere.radius*sphere.radius;
    float discriminant = half_b*half_b - a*c;

    if (discriminant > 0.0) {
        float root = sqrt(discriminant);
        float temp = (-half_b - root) / a;

        if (temp < dist_max && temp > dist_min) {
            record.dist = temp;
            record.point = Ray_position(ray, record.dist);
            vec3 outward_normal = (record.point - sphere.center) / sphere.radius;
            HitRecord_set_face_normal(record, ray, outward_normal);
            record.material_type = sphere.material_type;
            record.albedo = sphere.albedo;
            record.fuzziness = sphere.fuzziness;
            record.index_of_refraction = sphere.index_of_refraction;

            return true;
        }

        temp = (-half_b + root) / a;
        if (temp < dist_max && temp > dist_min) {
            record.dist = temp;
            record.point = Ray_position(ray, record.dist);
            vec3 outward_normal = (record.point - sphere.center) / sphere.radius;
            HitRecord_set_face_normal(record, ray, outward_normal);
            record.material_type = sphere.material_type;
            record.albedo = sphere.albedo;
            record.fuzziness = sphere.fuzziness;
            record.index_of_refraction = sphere.index_of_refraction;

            return true;
        }
    }
    return false;
}
// END SHAPES //

bool scene(Ray ray, float dist_min, float dist_max, out HitRecord record) {
    const float GLASS_ETA = 1.5;

    Sphere sphere_small = Sphere(vec3(0, 0, -1), 0.5, LAMBERTIAN, vec3(0.1, 0.5, 0.2), 0, 0);
    Sphere sphere_big   = Sphere(vec3(0, -100.5, -1), 100, LAMBERTIAN, vec3(0.2, 0.2, 0.8), 0, 0);

    Sphere lamb_1 = Sphere(vec3(0.7, 0.5, 0.5), 1, LAMBERTIAN, vec3(0.2, 0., 0.5), 0, 0);

    Sphere metal_1 = Sphere(vec3(-1.2, 0, 0.8), 0.5, METAL, vec3(0.8, 0.6, 0.2), 0.0, 0);
    Sphere metal_2 = Sphere(vec3(-1, 0, -3), 0.5, METAL, vec3(0.8, 0.8, 0.8), 0.5, 0);

    Sphere glass_2 = Sphere(vec3(-1.2, 0, -1), 0.5, DIELECTRIC, vec3(0), 0, 1.5);
    Sphere glass_3 = Sphere(vec3(-1.2, 0, -1),-0.45, DIELECTRIC, vec3(0), 0, 1.5);
    Sphere glass_big = Sphere(vec3(1.2, 0.5, -2), 1., DIELECTRIC, vec3(0), 0, 1.5);
    

    Sphere scene[] = Sphere[](sphere_small, sphere_big, lamb_1, metal_1, metal_2, glass_2, glass_3, glass_big);

    HitRecord temp_record;
    bool hit = false;
    float closest = dist_max;

    for (int i = 0; i < scene.length(); ++i) {
        if (Sphere_hit(scene[i], ray, dist_min, closest, temp_record)) {
            hit = true;
            closest = temp_record.dist;
            record = temp_record;
        }
    }

    return hit;
}

vec3 fire_ray(Ray ray) {
    HitRecord record;

    Ray scattered;
    vec3 attenuation;
    vec3 color = vec3(1);

    for(uint depth = 0; depth < max_ray_bounces; ++depth) {
        if (scene(ray, 0.001, MAX_FLOAT, record)) {
            if (Material_scatter(ray, record, attenuation, scattered)) {
                ray.origin = scattered.origin;
                ray.direction = scattered.direction;
                color *= attenuation;
            } else {
                color *= 0;
                break;
            }
        } else {
            float t = 0.5 * (normalize(ray.direction).y + 1.);
            color *= (1 - t)*vec3(1) + t*vec3(0.5, 0.7, 1.0);
            break;
        }
    }

    return color;
}

void main() {
    rand_state = (gl_FragCoord.xy / window_size) + sample_number * 15.23;

    vec3 position = vec3(-3.5, 2.5, 3);
    vec3 lookat = vec3(0, 0, -1);
    float focus_distance = length(position - lookat);

    Camera camera = Camera_new(
        position,                   // Position
        lookat,                     // Lookat
        vec3(0., 1., 0.),           // Up direction
        120.,                       // Vertical field of view (degrees)
        window_size.x/window_size.y,// Aspect ratio
        0.3,                        // Aperature size
        focus_distance              // Distance to focus
    );

    vec3 color = vec3(0);

    for (uint s = 0; s < samples_per_pixel; ++s) {
        vec2 uv = ((gl_FragCoord.xy + vec2(random(), random())) / window_size);
        // FIXME: y is flipped in this implementation for some reason (compared to compute shader)
        uv.y = 1 - uv.y;

        color += fire_ray(Camera_get_ray(camera, uv));
    }
    color /= float(samples_per_pixel);

    // Gamma correction & contrast adjustment
    color = smoothstep(0., 1., sqrt(color));

    // Global work group position (corresponds to current pixel in this case)
    // `imageStore` expects an ivec2, hence the cast
    ivec2 pixel_coords = ivec2(gl_FragCoord.xy);

    if (sample_number > 1) {
        color += imageLoad(img_output, pixel_coords).rgb;
    }
    imageStore(img_output, pixel_coords, vec4(color, sample_number));

    out_color = vec4(color / float(sample_number), 1.);
}