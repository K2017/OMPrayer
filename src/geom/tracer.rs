use crate::ray::Ray;
use super::*;
use crate::vec::*;
use crate::texture::Texture as _;

pub fn trace(r: &Ray, scene: &Scene, depth: usize) -> Vec3 {
    if depth == 0 {
        return glm::zero();
    }
    if let Some(TraceResult { material, hit }) = scene.trace(r, 0.001, std::f32::MAX) {
        let RayHit { normal, uv, .. } = hit;
        let w0 = -r.direction;
        let (bounce, pdf) = material.bounce(&w0, &hit);
        let incident = trace(&bounce, scene, depth - 1);
        let (brdf, ks) = material.brdf(&w0, &bounce.direction, &normal, uv);
        let specular = brdf / pdf;
        let diffuse = {
            let lambert = material.albedo.sample(uv) / glm::pi::<f32>();
            let kd = (glm::vec3(1.0, 1.0, 1.0) - ks) * (1.0 - material.metalness.sample(uv));
            let pdf = glm::one_over_two_pi::<f32>();
            kd.component_mul(&lambert) / pdf
        };
        let costheta = f32::max(glm::dot(&normal, &bounce.direction), 0.0);
        (diffuse + specular).component_mul(&incident) * costheta + material.emission.sample(uv)
    } else {
        let dir = r.direction.normalize();
        scene.environment.sample(Sphere::uv_at_dir(&dir))
    }
}
