use crate::aabb::*;
use crate::mesh::*;
use crate::picture::*;
use crate::zbuffer::*;

use std::f32::consts::PI;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct RenderOptions {
    pub view_pos: Vec3,
    pub light_pos: Vec3,
    pub light_color: Vec3,
    pub ambient_color: Vec3,
    pub model_color: Vec3,
    pub grid_color: Vec3,
    pub background_color: Vec4,
    pub zoom: f32,
    pub grid_visible: bool,
    pub draw_size_hint: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            view_pos: Vec3::new(-1.0, 1.0, -1.0).normalize(),
            light_pos: Vec3::new(-1.0, 0.5, -0.5),
            light_color: Vec3::new(0.6, 0.6, 0.6),
            ambient_color: Vec3::new(0.4, 0.4, 0.4),
            model_color: Vec3::new(0.0, 0.45, 1.0),
            grid_color: Vec3::new(0.1, 0.1, 0.1),
            background_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            grid_visible: true,
            zoom: 1.0,
            draw_size_hint: true,
        }
    }
}

#[derive(Debug)]
pub struct RasterBackend {
    pub render_options: RenderOptions,
    width: u32,
    height: u32,
    aspect_ratio: f32,
}

impl RasterBackend {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            render_options: RenderOptions::default(),
            width,
            height,
            aspect_ratio: width as f32 / height as f32,
        }
    }

    fn view_projection(&self, zoom: f32) -> Mat4 {
        // calculate view projection matrix
        let proj = glm::ortho(
            zoom * 0.5 * self.aspect_ratio,
            -zoom * 0.5 * self.aspect_ratio,
            -zoom * 0.5,
            zoom * 0.5,
            0.0,
            1.0,
        );
        let view = glm::look_at(
            &self.render_options.view_pos,
            &Vec3::new(0.0, 0.0, 0.0),
            &Vec3::new(0.0, 0.0, -1.0),
        );
        proj * view
    }

    pub fn fit_mesh_scale(&self, mesh: impl IntoIterator<Item = Triangle> + Copy) -> (AABB, f32) {
        let aabb = AABB::from_iterable(mesh);
        let vp = self.view_projection(1.0);

        // scale the model such that is fills the entire canvas
        (aabb, scale_for_unitsize(&vp, &aabb))
    }

    pub fn render(
        &self,
        mesh: impl IntoIterator<Item = Triangle> + Copy,
        model_scale: f32,
        aabb: &AABB,
        timeout: Option<Duration>,
    ) -> Picture {
        let start_time = Instant::now();

        let mut pic = Picture::new(self.width, self.height);
        let mut zbuf = ZBuffer::new(self.width, self.height);
        let mut scaled_aabb = *aabb;

        pic.fill(&(&self.render_options.background_color).into());

        let vp = self.view_projection(self.render_options.zoom);

        // calculate transforms taking the new model scale into account
        let model = Mat4::identity()
            .append_translation(&-aabb.center())
            .append_scaling(model_scale);
        let mvp = vp * model;

        // let the AABB match the transformed model
        scaled_aabb.apply_transform(&model);

        // eye normal pointing towards the camera in world space
        let eye_normal = self.render_options.view_pos.normalize();

        // grid in x and y direction
        if self.render_options.grid_visible {
            draw_grid(
                &mut pic,
                &vp,
                scaled_aabb.lower.z,
                &self.render_options.grid_color,
                aabb.size(),
                model_scale,
            );
            draw_grid(
                &mut pic,
                &(vp * glm::rotation(PI / 2.0, &Vec3::new(0.0, 0.0, 1.0))),
                scaled_aabb.lower.z,
                &self.render_options.grid_color,
                aabb.size(),
                model_scale,
            );
        }

        for t in mesh {
            // timed out?
            if let Some(timeout) = timeout {
                let dt = Instant::now() - start_time;
                if dt > timeout {
                    // abort
                    println!("... timeout!");
                    return pic;
                }
            }

            let normal = -t.normal;

            // backface culling
            if glm::dot(&eye_normal, &normal) < 0.0 {
                continue;
            }

            let v = &t.vertices;

            let v0 = matmul(&mvp, &v[0]);
            let v1 = matmul(&mvp, &v[1]);
            let v2 = matmul(&mvp, &v[2]);

            let v0m = matmul(&model, &v[0]);
            let v1m = matmul(&model, &v[1]);
            let v2m = matmul(&model, &v[2]);

            // triangle bounding box
            let min_x = v0.x.min(v1.x).min(v2.x);
            let min_y = v0.y.min(v1.y).min(v2.y);
            let max_x = v0.x.max(v1.x).max(v2.x);
            let max_y = v0.y.max(v1.y).max(v2.y);

            // triangle bounding box in screen space
            let smin_x = 0.max(((min_x + 1.0) / 2.0 * pic.width() as f32) as u32);
            let smin_y = 0.max(((min_y + 1.0) / 2.0 * pic.height() as f32) as u32);
            let smax_x = 0.max(pic.width().min(((max_x + 1.0) / 2.0 * pic.width() as f32) as u32));
            let smax_y = 0.max(pic.height().min(((max_y + 1.0) / 2.0 * pic.height() as f32) as u32));

            for y in smin_y..=smax_y {
                for x in smin_x..=smax_x {
                    // normalized screen coordinates [-1,1]
                    let nx = 2.0 * ((x as f32 / pic.width() as f32) - 0.5);
                    let ny = 2.0 * ((y as f32 / pic.height() as f32) - 0.5);

                    let p = Vec2::new(nx, ny);
                    let p0 = v0.xy();
                    let p1 = v1.xy();
                    let p2 = v2.xy();

                    let inside =
                        edge_fn(&p, &p0, &p1) <= 0.0 && edge_fn(&p, &p1, &p2) <= 0.0 && edge_fn(&p, &p2, &p0) <= 0.0;

                    if inside {
                        // calculate barycentric coordinates
                        let area = edge_fn(&p0, &p1, &p2);
                        let w0 = edge_fn(&p1, &p2, &p) / area;
                        let w1 = edge_fn(&p2, &p0, &p) / area;
                        let w2 = edge_fn(&p0, &p1, &p) / area;

                        // fragment position in screen space
                        let frag_pos = Vec3::new(
                            w0 * v0.x + w1 * v1.x + w2 * v2.x,
                            w0 * v0.y + w1 * v1.y + w2 * v2.y,
                            w0 * v0.z + w1 * v1.z + w2 * v2.z,
                        );

                        // fragment position in world space
                        let fp = Vec3::new(
                            w0 * v0m.x + w1 * v1m.x + w2 * v2m.x,
                            w0 * v0m.y + w1 * v1m.y + w2 * v2m.y,
                            w0 * v0m.z + w1 * v1m.z + w2 * v2m.z,
                        );

                        //let fp = matmul(&mvp_inv, &frag_pos);

                        if zbuf.test_and_set(x, y, frag_pos.z) {
                            // calculate lightning
                            let light_normal = (self.render_options.light_pos - fp).normalize(); // normal frag pos to light (world space)
                            let view_normal = (self.render_options.view_pos - fp).normalize(); // normal frag pos to view (world space)
                            let reflect_dir = glm::reflect_vec(&-light_normal, &normal);

                            // diffuse
                            let diff_color =
                                glm::dot(&normal, &light_normal).max(0.0) * self.render_options.light_color * 1.0;

                            // specular
                            let spec_color = (glm::dot(&view_normal, &reflect_dir).powf(16.0) * 0.7)
                                * self.render_options.light_color;

                            // merge
                            let mut color = self.render_options.ambient_color + diff_color + spec_color;
                            color.x *= self.render_options.model_color.x;
                            color.y *= self.render_options.model_color.y;
                            color.z *= self.render_options.model_color.z;

                            pic.set(x, y, &(color.x, color.y, color.z, 1.0).into());
                        }
                    }
                }
            }
        }

        if self.render_options.draw_size_hint {
            let margin = 3;
            let text_to_height_ratio = 16;

            let text = format!(
                "{}x{}x{}",
                aabb.size().x as i32,
                aabb.size().y as i32,
                aabb.size().z as i32
            );

            let text_size = pic.height() / text_to_height_ratio;

            pic.fill_rect(
                0,
                pic.height() as i32 - (text_size + margin * 2) as i32,
                pic.width() as i32,
                pic.height() as i32,
                &"333333FF".into(),
            );

            pic.stroke_string(
                margin,
                pic.height() - text_size - margin,
                &text,
                text_size as f32,
                &"FFFFFFFF".into(),
            );
        }

        pic
    }
}

fn edge_fn(a: &Vec2, b: &Vec2, c: &Vec2) -> f32 {
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}

fn scale_for_unitsize(mvp: &Mat4, aabb: &AABB) -> f32 {
    let edges = [
        //(mvp * Vec3::new(aabb.lower.x, aabb.lower.y, aabb.lower.z).to_homogeneous()).xyz(),
        matmul(mvp, &Vec3::new(aabb.lower.x, aabb.lower.y, aabb.lower.z)),
        matmul(mvp, &Vec3::new(aabb.upper.x, aabb.lower.y, aabb.lower.z)),
        matmul(mvp, &Vec3::new(aabb.lower.x, aabb.upper.y, aabb.lower.z)),
        matmul(mvp, &Vec3::new(aabb.upper.x, aabb.upper.y, aabb.lower.z)),
        matmul(mvp, &Vec3::new(aabb.lower.x, aabb.lower.y, aabb.upper.z)),
        matmul(mvp, &Vec3::new(aabb.upper.x, aabb.lower.y, aabb.upper.z)),
        matmul(mvp, &Vec3::new(aabb.lower.x, aabb.upper.y, aabb.upper.z)),
        matmul(mvp, &Vec3::new(aabb.upper.x, aabb.upper.y, aabb.upper.z)),
    ];

    let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

    for e in &edges {
        min.x = min.x.min(e.x);
        min.y = min.y.min(e.y);

        max.x = max.x.max(e.x);
        max.y = max.y.max(e.y);
    }

    1.0 / ((f32::abs(max.x - min.x)).max(f32::abs(max.y - min.y)) / 2.0)
}

fn draw_grid(pic: &mut Picture, vp: &Mat4, z: f32, color: &Vec3, model_size: Vec3, scale: f32) {
    // draw grid
    let max_xy = model_size.x.max(model_size.y);
    let grid_color = (color.x, color.y, color.z, 1.0).into();
    let grid_size = 10.0; // mm
    let grid_count = ((max_xy * scale) / scale / grid_size + 1.0) as i32;
    let grid_spacing = grid_size * scale;

    let ox = grid_count as f32 * grid_spacing / 2.0;

    for x in 0..=grid_count {
        let p0 = Vec3::new(grid_spacing * x as f32 - ox, grid_count as f32 * grid_spacing * 0.5, z);
        let p1 = Vec3::new(p0.x, -grid_count as f32 * grid_spacing * 0.5, z);

        // to screen space
        let sp0 = matmul(vp, &p0).xy();
        let sp1 = matmul(vp, &p1).xy();

        pic.thick_line(
            ((sp0.x + 1.0) / 2.0 * pic.width() as f32) as i32,
            ((sp0.y + 1.0) / 2.0 * pic.height() as f32) as i32,
            ((sp1.x + 1.0) / 2.0 * pic.width() as f32) as i32,
            ((sp1.y + 1.0) / 2.0 * pic.height() as f32) as i32,
            &grid_color,
            1.0,
        );
    }
}
