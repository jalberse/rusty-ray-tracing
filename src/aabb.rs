use glam::DVec3;

use crate::ray::Ray;

const DIMENSIONS: usize = 3;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Aabb {
    min: DVec3,
    max: DVec3,
}

impl Aabb {
    pub fn new(min: DVec3, max: DVec3) -> Aabb {
        Aabb { min, max }
    }

    pub fn min(&self) -> &DVec3 {
        &self.min
    }

    pub fn max(&self) -> &DVec3 {
        &self.max
    }

    /// Returns true iff the ray intersects the bounding box;
    /// follows Andrew Kensler's hit method.
    pub fn hit(&self, ray: &Ray, mut t_min: f64, mut t_max: f64) -> bool {
        for i in 0..DIMENSIONS {
            let inv_d = 1.0 / ray.direction[i];
            let t0 = (self.min[i] - ray.origin[i]) * inv_d;
            let t1 = (self.max[i] - ray.origin[i]) * inv_d;
            let (t0, t1) = if inv_d < 0.0 { (t1, t0) } else { (t0, t1) };
            t_min = if t0 > t_min { t0 } else { t_min };
            t_max = if t1 < t_max { t1 } else { t_max };
            if t_max < t_min {
                return false;
            }
        }
        true
    }

    pub fn union(box0: &Option<Aabb>, box1: &Option<Aabb>) -> Option<Aabb> {
        if let Some(bb0) = box0 {
            if let Some(bb1) = box1 {
                let min = DVec3::new(
                    f64::min(bb0.min().x, bb1.min().x),
                    f64::min(bb0.min().y, bb1.min().y),
                    f64::min(bb0.min().z, bb1.min().z),
                );
                let max = DVec3::new(
                    f64::max(bb0.max().x, bb1.max().x),
                    f64::max(bb0.max().y, bb1.max().y),
                    f64::max(bb0.max().z, bb1.max().z),
                );
                return Some(Aabb::new(min, max));
            } else {
                return box0.clone();
            }
        } else {
            if box1.is_some() {
                return box1.clone();
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use glam::DVec3;

    use crate::ray::Ray;

    use super::Aabb;

    #[test]
    fn hits() {
        let origin = DVec3::ZERO;
        let ray = Ray::new(origin, DVec3::Z);

        let min = DVec3::new(-1.0, -1.0, 1.0);
        let max = DVec3::new(1.0, 1.0, 2.0);

        let aabb = Aabb::new(min, max);

        assert!(aabb.hit(&ray, 0.0, 5.0));
    }

    #[test]
    fn misses() {
        let origin = DVec3::ZERO;
        let ray = Ray::new(origin, DVec3::Z);

        let min = DVec3::new(1.0, 1.0, 1.0);
        let max = DVec3::new(2.0, 2.0, 2.0);

        let aabb = Aabb::new(min, max);

        assert!(!aabb.hit(&ray, 0.0, 5.0));
    }

    #[test]
    fn union_nones() {
        assert!(Aabb::union(&None, &None).is_none());
    }

    #[test]
    fn union_box_0_some_other_none() {
        let min = DVec3::new(1.0, 1.0, 1.0);
        let max = DVec3::new(2.0, 2.0, 2.0);
        let aabb = Aabb::new(min, max);

        assert_eq!(Some(aabb), Aabb::union(&Some(aabb), &None));
    }

    #[test]
    fn union_box_1_some_other_none() {
        let min = DVec3::new(1.0, 1.0, 1.0);
        let max = DVec3::new(2.0, 2.0, 2.0);
        let aabb = Aabb::new(min, max);

        assert_eq!(Some(aabb), Aabb::union(&None, &Some(aabb)));
    }

    #[test]
    fn union() {
        let min_0 = DVec3::new(0.0, 1.0, 0.0);
        let max_0 = DVec3::new(2.0, 4.0, 2.0);
        let aabb_0 = Aabb::new(min_0, max_0);

        let min_1 = DVec3::new(1.0, 0.0, 1.0);
        let max_1 = DVec3::new(3.0, 3.0, 3.0);
        let aabb_1 = Aabb::new(min_1, max_1);

        let expected_min = DVec3::new(0.0, 0.0, 0.0);
        let expected_max = DVec3::new(3.0, 4.0, 3.0);
        let expected_aabb = Aabb::new(expected_min, expected_max);

        assert_eq!(
            Some(expected_aabb),
            Aabb::union(&Some(aabb_0), &Some(aabb_1))
        );
    }
}