use bevy::prelude::*;
use crate::map::{Map, Sector, Wall};

/// A `CurrentSector` component tracks which Build engine sector an entity is currently in.
#[derive(Component, Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct CurrentSector(pub i16); // -1 = unknown

/// Runtime spatial index of the parsed Build engine map.
/// Provides sector queries equivalent to ENGINE.C utility functions.
#[derive(Resource, Debug, Clone)]
pub struct SectorMap {
    pub sectors: Vec<Sector>,
    pub walls: Vec<Wall>,
    // Pre-computed neighbor lists for fast sector traversal
    sector_neighbors: Vec<Vec<usize>>,
    pub base_elevations: Vec<(i32, i32)>,
}

impl SectorMap {
    pub fn from_map(map: &Map) -> Self {
        let mut sector_neighbors = Vec::with_capacity(map.sectors.len());
        for sector in &map.sectors {
            let mut neighbors = Vec::new();
            let start = sector.wallptr as usize;
            let end = start + sector.wallnum as usize;
            for i in start..end {
                if let Some(wall) = map.walls.get(i) {
                    if wall.nextsector >= 0 {
                        let next_sec = wall.nextsector as usize;
                        if !neighbors.contains(&next_sec) {
                            neighbors.push(next_sec);
                        }
                    }
                }
            }
            sector_neighbors.push(neighbors);
        }

        let base_elevations = map.sectors.iter().map(|s| (s.floorz, s.ceilingz)).collect();

        Self {
            sectors: map.sectors.clone(),
            walls: map.walls.clone(),
            sector_neighbors,
            base_elevations,
        }
    }

    pub fn get_base_floorz(&self, idx: usize) -> i32 {
        self.base_elevations
            .get(idx)
            .map(|&(f, _)| f)
            .unwrap_or_else(|| self.sectors.get(idx).map(|s| s.floorz).unwrap_or(0))
    }

    pub fn get_base_ceilingz(&self, idx: usize) -> i32 {
        self.base_elevations
            .get(idx)
            .map(|&(_, c)| c)
            .unwrap_or_else(|| self.sectors.get(idx).map(|s| s.ceilingz).unwrap_or(0))
    }

    pub fn point_in_sector(&self, build_x: i32, build_y: i32, sector_idx: usize) -> bool {
        if let Some(sector) = self.sectors.get(sector_idx) {
            let mut inside = false;
            let start = sector.wallptr as usize;
            let end = start + sector.wallnum as usize;
            
            for i in start..end {
                if let Some(wall) = self.walls.get(i) {
                    if let Some(point2_wall) = self.walls.get(wall.point2 as usize) {
                        let x1 = wall.x;
                        let y1 = wall.y;
                        let x2 = point2_wall.x;
                        let y2 = point2_wall.y;

                        // Ray-casting algorithm: count crossings with ray from (build_x, build_y) in +x direction
                        if (y1 > build_y) != (y2 > build_y) {
                            let intersect_x = (x2 - x1) as f64 * (build_y - y1) as f64 / (y2 - y1) as f64 + x1 as f64;
                            if (build_x as f64) < intersect_x {
                                inside = !inside;
                            }
                        }
                    }
                }
            }
            inside
        } else {
            false
        }
    }

    pub fn find_sector(&self, build_x: i32, build_y: i32, hint: Option<usize>) -> Option<usize> {
        if let Some(hint_idx) = hint {
            if self.point_in_sector(build_x, build_y, hint_idx) {
                return Some(hint_idx);
            }
            
            if let Some(neighbors) = self.sector_neighbors.get(hint_idx) {
                for &neighbor_idx in neighbors {
                    if self.point_in_sector(build_x, build_y, neighbor_idx) {
                        return Some(neighbor_idx);
                    }
                }
            }
        }
        
        // Brute force fallback
        for (idx, _) in self.sectors.iter().enumerate() {
            // Skip the ones we already checked
            if let Some(hint_idx) = hint {
                if idx == hint_idx { continue; }
                if let Some(neighbors) = self.sector_neighbors.get(hint_idx) {
                    if neighbors.contains(&idx) { continue; }
                }
            }
            
            if self.point_in_sector(build_x, build_y, idx) {
                return Some(idx);
            }
        }
        
        None
    }

    pub fn find_sector_world(&self, world_x: f32, world_z: f32, hint: Option<usize>) -> Option<usize> {
        let build_x = (world_x * 1024.0) as i32;
        let build_y = (world_z * 1024.0) as i32;
        self.find_sector(build_x, build_y, hint)
    }

    pub fn get_floor_y_at(&self, sector_idx: usize, world_x: f32, world_z: f32) -> f32 {
        if let Some(sector) = self.sectors.get(sector_idx) {
            let build_x = (world_x * 1024.0) as i32;
            let build_y = (world_z * 1024.0) as i32;
            sector.get_floor_y_at(&self.walls, build_x, build_y)
        } else {
            0.0
        }
    }

    pub fn get_ceil_y_at(&self, sector_idx: usize, world_x: f32, world_z: f32) -> f32 {
        if let Some(sector) = self.sectors.get(sector_idx) {
            let build_x = (world_x * 1024.0) as i32;
            let build_y = (world_z * 1024.0) as i32;
            sector.get_ceiling_y_at(&self.walls, build_x, build_y)
        } else {
            0.0
        }
    }

    pub fn get_sector_lotag(&self, sector_idx: usize) -> i16 {
        self.sectors.get(sector_idx).map(|s| s.lotag).unwrap_or(0)
    }

    pub fn get_sector_ceilingstat(&self, sector_idx: usize) -> i16 {
        self.sectors.get(sector_idx).map(|s| s.ceilingstat).unwrap_or(0)
    }
}

pub fn update_entity_sectors(
    sector_map: Option<Res<SectorMap>>,
    mut query: Query<(&Transform, &mut CurrentSector)>,
) {
    if let Some(map) = sector_map {
        for (transform, mut current_sector) in query.iter_mut() {
            let hint = if current_sector.0 >= 0 {
                Some(current_sector.0 as usize)
            } else {
                None
            };
            
            if let Some(new_sector) = map.find_sector_world(transform.translation.x, transform.translation.z, hint) {
                if current_sector.0 != new_sector as i16 {
                    current_sector.0 = new_sector as i16;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_wall(x: i32, y: i32, point2: i16, nextsector: i16, nextwall: i16) -> Wall {
        Wall {
            x, y, point2, nextwall, nextsector,
            cstat: 0, picnum: 0, overpicnum: 0, shade: 0, pal: 0,
            xrepeat: 8, yrepeat: 8, xpanning: 0, ypanning: 0,
            lotag: 0, hitag: 0, extra: 0,
        }
    }

    fn make_sector(wallptr: i16, wallnum: i16, lotag: i16) -> Sector {
        Sector {
            wallptr, wallnum,
            ceilingz: -16384, floorz: 0,
            ceilingstat: 0, floorstat: 0,
            ceilingpicnum: 0, ceilingheinum: 0,
            ceilingshade: 0, ceilingpal: 0,
            ceilingxpanning: 0, ceilingypanning: 0,
            floorpicnum: 0, floorheinum: 0,
            floorshade: 0, floorpal: 0,
            floorxpanning: 0, floorypanning: 0,
            visibility: 0, _filler: 0,
            lotag, hitag: 0, extra: 0,
        }
    }

    fn create_test_map() -> Map {
        // Simple square sector: (0,0) to (1024,1024)
        let walls = vec![
            make_wall(0, 0, 1, -1, -1),
            make_wall(1024, 0, 2, -1, -1),
            make_wall(1024, 1024, 3, -1, -1),
            make_wall(0, 1024, 0, -1, -1),
        ];

        Map {
            version: 7, posx: 512, posy: 512, posz: 0, ang: 0, cursectnum: 0,
            sectors: vec![make_sector(0, 4, 0)],
            walls,
            sprites: vec![],
        }
    }

    fn create_two_sector_map() -> Map {
        // Sector 0: (0,0) to (1024,1024) — shares right wall with sector 1
        // Sector 1: (1024,0) to (2048,1024)
        let walls = vec![
            // Sector 0 walls (indices 0-3)
            make_wall(0, 0, 1, -1, -1),
            make_wall(1024, 0, 2, 1, 5),     // portal to sector 1
            make_wall(1024, 1024, 3, -1, -1),
            make_wall(0, 1024, 0, -1, -1),
            // Sector 1 walls (indices 4-7)
            make_wall(1024, 0, 5, -1, -1),
            make_wall(2048, 0, 6, 0, 1),      // portal back to sector 0
            make_wall(2048, 1024, 7, -1, -1),
            make_wall(1024, 1024, 4, -1, -1),
        ];

        Map {
            version: 7, posx: 512, posy: 512, posz: 0, ang: 0, cursectnum: 0,
            sectors: vec![
                make_sector(0, 4, 0),
                make_sector(4, 4, 2),  // lotag 2 = underwater
            ],
            walls,
            sprites: vec![],
        }
    }

    #[test]
    fn test_point_in_sector() {
        let map = create_test_map();
        let sector_map = SectorMap::from_map(&map);

        assert!(sector_map.point_in_sector(512, 512, 0));
        assert!(!sector_map.point_in_sector(-10, 512, 0));
        assert!(!sector_map.point_in_sector(1050, 512, 0));
    }

    #[test]
    fn test_find_sector() {
        let map = create_test_map();
        let sector_map = SectorMap::from_map(&map);

        assert_eq!(sector_map.find_sector(512, 512, None), Some(0));
        assert_eq!(sector_map.find_sector(-10, 512, None), None);
    }

    #[test]
    fn test_find_sector_with_hint_and_neighbors() {
        let map = create_two_sector_map();
        let sector_map = SectorMap::from_map(&map);

        // Point in sector 0 — found directly from hint
        assert_eq!(sector_map.find_sector(512, 512, Some(0)), Some(0));
        // Point in sector 1 — found via neighbor from hint=0
        assert_eq!(sector_map.find_sector(1536, 512, Some(0)), Some(1));
        // Point in sector 0 — found via neighbor from hint=1
        assert_eq!(sector_map.find_sector(512, 512, Some(1)), Some(0));
        // Outside both sectors
        assert_eq!(sector_map.find_sector(3000, 512, Some(0)), None);
    }

    #[test]
    fn test_sector_lotag_query() {
        let map = create_two_sector_map();
        let sector_map = SectorMap::from_map(&map);

        assert_eq!(sector_map.get_sector_lotag(0), 0);
        assert_eq!(sector_map.get_sector_lotag(1), 2); // underwater
        assert_eq!(sector_map.get_sector_lotag(999), 0); // out of bounds
    }

    #[test]
    fn test_coordinate_conversion() {
        let map = create_test_map();
        let sector_map = SectorMap::from_map(&map);

        // world_x = 0.5 -> build_x = 512
        // world_z = 0.5 -> build_y = 512
        assert_eq!(sector_map.find_sector_world(0.5, 0.5, None), Some(0));
        assert_eq!(sector_map.find_sector_world(-0.1, 0.5, None), None);
    }

    #[test]
    fn test_raycast_precision_in_negative_coords() {
        let walls = vec![
            make_wall(-4, 0, 1, -1, -1),
            make_wall(-2, 10, 2, -1, -1),
            make_wall(10, 10, 3, -1, -1),
            make_wall(10, 0, 0, -1, -1),
        ];
        let map = Map {
            version: 7, posx: 0, posy: 0, posz: 0, ang: 0, cursectnum: 0,
            sectors: vec![make_sector(0, 4, 0)],
            walls,
            sprites: vec![],
        };
        let sector_map = SectorMap::from_map(&map);

        // At y = 4, the left edge connects (-4, 0) to (-2, 10), so x_intersect = -3.2.
        // Point (-4, 4) is to the left of the left edge (outside).
        // A ray to +inf crosses left edge at -3.2 and right edge at 10 (2 crossings -> even -> outside).
        assert!(!sector_map.point_in_sector(-4, 4, 0));

        // Point (-3, 4) is inside the polygon (x is between -3.2 and 10).
        // Ray crosses only the right edge at 10 (1 crossing -> odd -> inside).
        assert!(sector_map.point_in_sector(-3, 4, 0));

        // Point (0, 4) is well inside.
        assert!(sector_map.point_in_sector(0, 4, 0));

        // Point (11, 4) is to the right of the polygon (outside).
        assert!(!sector_map.point_in_sector(11, 4, 0));
    }

    #[test]
    fn test_sloped_height_queries_in_sector_map() {
        let mut sector = make_sector(0, 4, 0);
        sector.floorstat = 2; // sloped floor
        sector.floorheinum = 1024;
        sector.ceilingstat = 2; // sloped ceiling
        sector.ceilingheinum = 512;
        sector.ceilingz = -16384;
        sector.floorz = 0;

        let walls = vec![
            make_wall(0, 0, 1, -1, -1),
            make_wall(1024, 0, 2, -1, -1),
            make_wall(1024, 1024, 3, -1, -1),
            make_wall(0, 1024, 0, -1, -1),
        ];

        let map = Map {
            version: 7, posx: 0, posy: 0, posz: 0, ang: 0, cursectnum: 0,
            sectors: vec![sector],
            walls,
            sprites: vec![],
        };
        let sector_map = SectorMap::from_map(&map);

        // At world_x = 0.5, world_z = 0.0 (build y = 0), on pivot wall:
        // floor z = 0 -> floor y = 0.0
        assert_eq!(sector_map.get_floor_y_at(0, 0.5, 0.0), 0.0);

        // At world_x = 0.5, world_z = 0.5 (build y = 512):
        // z_offset = (1024 * 512) / 256 = 2048
        // floor_y = -2048 / 16384 = -0.125
        assert_eq!(sector_map.get_floor_y_at(0, 0.5, 0.5), -0.125);

        // Ceiling query: ceilingz = -16384, heinum = 512
        // at y = 0: ceil_y = -(-16384) / 16384 = 1.0
        assert_eq!(sector_map.get_ceil_y_at(0, 0.5, 0.0), 1.0);
        // at y = 512: z_offset = (512 * 512) / 256 = 1024 -> ceilingz = -15360 -> ceil_y = 15360 / 16384 = 0.9375
        assert_eq!(sector_map.get_ceil_y_at(0, 0.5, 0.5), 0.9375);
    }

    #[test]
    fn test_find_sector_fallback_when_hint_invalid() {
        let map = create_two_sector_map();
        let sector_map = SectorMap::from_map(&map);

        // Hint is 0, but entity teleported into sector 1 without being adjacent
        assert_eq!(sector_map.find_sector(1536, 512, Some(0)), Some(1));
        // Hint is 99 (invalid out of bounds hint)
        assert_eq!(sector_map.find_sector(512, 512, Some(99)), Some(0));
    }
}

