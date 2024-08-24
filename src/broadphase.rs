use crate::polygon::Polygon;
use std::ops::{Add, Sub};

#[cfg(feature = "bevy")]
use bevy::math::DVec3;

#[cfg(not(feature = "bevy"))]
use glam::DVec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RaycastResult {
    pub distance: f64,
    pub point: DVec3,
    pub normal: DVec3,
}

#[derive(Clone, Copy, Debug)]
pub struct Raycast {
    pub origin: DVec3,
    pub direction: DVec3,
}

impl Raycast {
    pub fn new(origin: DVec3, direction: DVec3) -> Self {
        Self { origin, direction }
    }

    pub fn cast_against_polygons(&self, polygons: &Vec<Polygon>) -> Option<RaycastResult> {
        let mut closest_result = None;
        let mut closest_distance = f64::INFINITY;

        for polygon in polygons {
            if let Some(result) = self.cast_against_polygon(polygon) {
                if result.distance < closest_distance {
                    closest_distance = result.distance;
                    closest_result = Some(result);
                }
            }
        }

        closest_result
    }

    fn cast_against_polygon(&self, polygon: &Polygon) -> Option<RaycastResult> {
        let normal = polygon.surface.normal;
        let denominator = normal.dot(self.direction);

        // Ignore backfaces: only consider polygons facing towards the ray
        if denominator >= 0.0 {
            return None;
        }

        let t = (polygon.vertices[0].pos - self.origin).dot(normal) / denominator;

        if t < 0.0 {
            return None;
        }

        let point = self.origin + self.direction * t;

        if !polygon.contains_point(point) {
            return None;
        }

        Some(RaycastResult {
            distance: t,
            point,
            normal,
        })
    }

    pub fn cast_against_aabb(&self, aabb: &Aabb) -> Option<RaycastResult> {
        let inv_direction = DVec3::splat(1.0) / self.direction;

        let t1 = (aabb.min - self.origin) * inv_direction;
        let t2 = (aabb.max - self.origin) * inv_direction;

        let t_min = t1.min(t2);
        let t_max = t1.max(t2);

        let t_enter = t_min.max_element();
        let t_exit = t_max.min_element();

        if t_exit < 0.0 || t_enter > t_exit {
            return None;
        }

        let distance = t_enter.max(0.0);
        let point = self.origin + self.direction * distance;

        let mut normal = DVec3::ZERO;
        if t_enter == t1.x {
            normal.x = -1.0 * inv_direction.x.signum();
        } else if t_enter == t2.x {
            normal.x = 1.0 * inv_direction.x.signum();
        } else if t_enter == t1.y {
            normal.y = -1.0 * inv_direction.y.signum();
        } else if t_enter == t2.y {
            normal.y = 1.0 * inv_direction.y.signum();
        } else if t_enter == t1.z {
            normal.z = -1.0 * inv_direction.z.signum();
        } else if t_enter == t2.z {
            normal.z = 1.0 * inv_direction.z.signum();
        }

        Some(RaycastResult {
            distance,
            point,
            normal,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Reflect))]
pub struct Aabb {
    pub min: DVec3,
    pub max: DVec3,
}

impl From<&Vec<Polygon>> for Aabb {
    fn from(polygons: &Vec<Polygon>) -> Self {
        let mut min = DVec3::splat(f64::INFINITY);
        let mut max = DVec3::splat(f64::NEG_INFINITY);

        for polygon in polygons {
            for vertex in &polygon.vertices {
                min = min.min(vertex.pos);
                max = max.max(vertex.pos);
            }
        }

        Aabb { min, max }
    }
}

impl Add<Aabb> for Aabb {
    type Output = Aabb;

    fn add(self, other: Aabb) -> Aabb {
        Aabb {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

impl Sub<Aabb> for Aabb {
    type Output = Aabb;

    fn sub(self, other: Aabb) -> Aabb {
        Aabb {
            min: self.min.max(other.min),
            max: self.max.min(other.max),
        }
    }
}

impl Aabb {
    pub fn new(min: DVec3, max: DVec3) -> Self {
        Self { min, max }
    }

    pub fn center(&self) -> DVec3 {
        (self.min + self.max) * 0.5
    }

    pub fn extents(&self) -> DVec3 {
        (self.max - self.min) * 0.5
    }

    pub fn contains(&self, point: DVec3) -> bool {
        point.x >= self.min.x
            && point.y >= self.min.y
            && point.z >= self.min.z
            && point.x <= self.max.x
            && point.y <= self.max.y
            && point.z <= self.max.z
    }

    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    pub fn surface_area(&self) -> f64 {
        let extents = self.extents();
        2.0 * (extents.x * extents.y + extents.x * extents.z + extents.y * extents.z)
    }
}

#[cfg(test)]
mod tests {
    use super::{Aabb, Raycast};

    #[cfg(feature = "bevy")]
    use bevy::math::DVec3;

    #[cfg(not(feature = "bevy"))]
    use glam::DVec3;

    #[test]
    fn test_aabb_contains() {
        let aabb = Aabb::new(DVec3::new(-1.0, -1.0, -1.0), DVec3::new(1.0, 1.0, 1.0));
        assert!(aabb.contains(DVec3::new(0.0, 0.0, 0.0)));
        assert!(!aabb.contains(DVec3::new(2.0, 0.0, 0.0)));
    }

    #[test]
    fn test_aabb_intersects() {
        let aabb1 = Aabb::new(DVec3::new(-1.0, -1.0, -1.0), DVec3::new(1.0, 1.0, 1.0));
        let aabb2 = Aabb::new(DVec3::new(0.0, 0.0, 0.0), DVec3::new(2.0, 2.0, 2.0));
        let aabb3 = Aabb::new(DVec3::new(2.0, 2.0, 2.0), DVec3::new(3.0, 3.0, 3.0));
        assert!(aabb1.intersects(&aabb2));
        assert!(!aabb1.intersects(&aabb3));
    }

    #[test]
    fn test_aabb_union() {
        let aabb1 = Aabb::new(DVec3::new(-1.0, -1.0, -1.0), DVec3::new(1.0, 1.0, 1.0));
        let aabb2 = Aabb::new(DVec3::new(0.0, 0.0, 0.0), DVec3::new(2.0, 2.0, 2.0));
        let union = aabb1 + aabb2;
        assert_eq!(union.min, DVec3::new(-1.0, -1.0, -1.0));
        assert_eq!(union.max, DVec3::new(2.0, 2.0, 2.0));
    }

    #[test]
    fn test_aabb_intersection() {
        let aabb1 = Aabb::new(DVec3::new(-1.0, -1.0, -1.0), DVec3::new(1.0, 1.0, 1.0));
        let aabb2 = Aabb::new(DVec3::new(0.0, 0.0, 0.0), DVec3::new(2.0, 2.0, 2.0));
        let intersection = aabb1 - aabb2;
        assert_eq!(intersection.min, DVec3::new(0.0, 0.0, 0.0));
        assert_eq!(intersection.max, DVec3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn test_raycast_hit() {
        let aabb = Aabb::new(DVec3::new(-1.0, -1.0, -1.0), DVec3::new(1.0, 1.0, 1.0));
        let raycast = Raycast::new(DVec3::new(0.0, 0.0, -2.0), DVec3::new(0.0, 0.0, 1.0));
        let result = raycast.cast_against_aabb(&aabb).unwrap();
        assert_eq!(result.distance, 1.0);
        assert_eq!(result.point, DVec3::new(0.0, 0.0, -1.0));
        assert_eq!(result.normal, DVec3::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn test_raycast_miss() {
        let aabb = Aabb::new(DVec3::new(-1.0, -1.0, -1.0), DVec3::new(1.0, 1.0, 1.0));
        let raycast = Raycast::new(DVec3::new(0.0, 0.0, -2.0), DVec3::new(0.0, 0.0, -1.0));
        let result = raycast.cast_against_aabb(&aabb);
        assert!(result.is_none());
    }
}
