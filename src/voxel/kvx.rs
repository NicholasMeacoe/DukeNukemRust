use bevy::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KvxError {
    UnexpectedEof,
    InvalidDimensions,
    InvalidOffset,
    CorruptedSlab,
}

impl std::fmt::Display for KvxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KvxError::UnexpectedEof => write!(f, "Unexpected end of file while parsing KVX"),
            KvxError::InvalidDimensions => write!(f, "Invalid dimensions in KVX header"),
            KvxError::InvalidOffset => write!(f, "Invalid slab or column offset in KVX"),
            KvxError::CorruptedSlab => write!(f, "Corrupted slab data in KVX column"),
        }
    }
}

impl std::error::Error for KvxError {}

/// A 3D voxel model parsed from Ken Silverman's Build engine KVX format.
#[derive(Debug, Clone, PartialEq)]
pub struct KvxModel {
    pub xsiz: usize,
    pub ysiz: usize,
    pub zsiz: usize,
    pub pivot: Vec3,
    pub voxels: Vec<Option<u8>>,
}

impl KvxModel {
    /// Creates an empty voxel model with specified dimensions and pivot.
    pub fn new(xsiz: usize, ysiz: usize, zsiz: usize, pivot: Vec3) -> Self {
        Self {
            xsiz,
            ysiz,
            zsiz,
            pivot,
            voxels: vec![None; xsiz * ysiz * zsiz],
        }
    }

    /// Internal linear index for (x, y, z) coordinate.
    #[inline]
    pub fn index(&self, x: usize, y: usize, z: usize) -> usize {
        x + y * self.xsiz + z * (self.xsiz * self.ysiz)
    }

    /// Returns the 8-bit palette color index at (x, y, z), or None if empty or out of bounds.
    #[inline]
    pub fn get_voxel(&self, x: usize, y: usize, z: usize) -> Option<u8> {
        if x < self.xsiz && y < self.ysiz && z < self.zsiz {
            self.voxels[self.index(x, y, z)]
        } else {
            None
        }
    }

    /// Sets the voxel color at (x, y, z).
    #[inline]
    pub fn set_voxel(&mut self, x: usize, y: usize, z: usize, color: Option<u8>) {
        if x < self.xsiz && y < self.ysiz && z < self.zsiz {
            let idx = self.index(x, y, z);
            self.voxels[idx] = color;
        }
    }

    /// Total number of solid voxels in the volume.
    pub fn solid_count(&self) -> usize {
        self.voxels.iter().filter(|v| v.is_some()).count()
    }

    /// Parses a raw binary `.KVX` byte slice into a `KvxModel`.
    pub fn parse(data: &[u8]) -> Result<Self, KvxError> {
        if data.len() < 28 {
            return Err(KvxError::UnexpectedEof);
        }

        let numbytes = i32::from_le_bytes(data[0..4].try_into().unwrap());
        let xsiz = i32::from_le_bytes(data[4..8].try_into().unwrap());
        let ysiz = i32::from_le_bytes(data[8..12].try_into().unwrap());
        let zsiz = i32::from_le_bytes(data[12..16].try_into().unwrap());
        let xpivot = i32::from_le_bytes(data[16..20].try_into().unwrap());
        let ypivot = i32::from_le_bytes(data[20..24].try_into().unwrap());
        let zpivot = i32::from_le_bytes(data[24..28].try_into().unwrap());

        if xsiz <= 0 || ysiz <= 0 || zsiz <= 0 || xsiz > 512 || ysiz > 512 || zsiz > 512 {
            return Err(KvxError::InvalidDimensions);
        }

        let ux = xsiz as usize;
        let uy = ysiz as usize;
        let uz = zsiz as usize;

        // Pivot in KVX is stored in 256ths (fixed point)
        let pivot = Vec3::new(
            xpivot as f32 / 256.0,
            ypivot as f32 / 256.0,
            zpivot as f32 / 256.0,
        );

        let mut model = KvxModel::new(ux, uy, uz, pivot);

        let xoffset_start = 28;
        let xoffset_len = (ux + 1) * 4;
        let xyoffset_start = xoffset_start + xoffset_len;
        let xyoffset_len = ux * (uy + 1) * 2;
        let slab_data_start = xyoffset_start + xyoffset_len;

        if data.len() < slab_data_start {
            return Err(KvxError::UnexpectedEof);
        }

        // Parse xoffsets
        let mut xoffsets = Vec::with_capacity(ux + 1);
        for i in 0..=ux {
            let off = xoffset_start + i * 4;
            let val = u32::from_le_bytes(data[off..off + 4].try_into().unwrap()) as usize;
            xoffsets.push(val);
        }

        // Parse each (x, y) column
        for x in 0..ux {
            for y in 0..uy {
                let xy_idx = x * (uy + 1) + y;
                let xy_off = xyoffset_start + xy_idx * 2;
                let start_col = u16::from_le_bytes(data[xy_off..xy_off + 2].try_into().unwrap()) as usize;
                let end_col = u16::from_le_bytes(data[xy_off + 2..xy_off + 4].try_into().unwrap()) as usize;

                let abs_start = slab_data_start + start_col;
                let abs_end = slab_data_start + end_col;

                if abs_start > data.len() || abs_end > data.len() || abs_start > abs_end {
                    continue;
                }

                let mut curr = abs_start;
                while curr + 3 <= abs_end {
                    let ztop = data[curr] as usize;
                    let collen = data[curr + 1] as usize;
                    let _flags = data[curr + 2];
                    curr += 3;

                    if collen == 0 {
                        break;
                    }

                    if curr + collen > abs_end {
                        break;
                    }

                    for z_i in 0..collen {
                        let z = ztop + z_i;
                        if z < uz {
                            model.set_voxel(x, y, z, Some(data[curr + z_i]));
                        }
                    }
                    curr += collen;
                }
            }
        }

        let _ = numbytes;
        Ok(model)
    }

    /// Serializes the model into authentic Ken Silverman Build `.KVX` bytes.
    pub fn encode(&self) -> Vec<u8> {
        let mut slab_bytes = Vec::new();
        let mut xyoffsets = Vec::with_capacity(self.xsiz * (self.ysiz + 1));

        for x in 0..self.xsiz {
            for y in 0..self.ysiz {
                xyoffsets.push(slab_bytes.len() as u16);

                // Compress column into continuous slabs
                let mut z = 0;
                while z < self.zsiz {
                    if let Some(color) = self.get_voxel(x, y, z) {
                        let ztop = z;
                        let mut slab_colors = vec![color];
                        z += 1;
                        while z < self.zsiz {
                            if let Some(next_c) = self.get_voxel(x, y, z) {
                                slab_colors.push(next_c);
                                z += 1;
                            } else {
                                break;
                            }
                        }

                        slab_bytes.push(ztop as u8);
                        slab_bytes.push(slab_colors.len() as u8);
                        slab_bytes.push(0); // flags
                        slab_bytes.extend_from_slice(&slab_colors);
                    } else {
                        z += 1;
                    }
                }
            }
            // End offset for the last column of this X plane
            xyoffsets.push(slab_bytes.len() as u16);
        }

        let xoffset_len = (self.xsiz + 1) * 4;
        let xyoffset_len = self.xsiz * (self.ysiz + 1) * 2;
        let total_size = 28 + xoffset_len + xyoffset_len + slab_bytes.len();

        let mut out = Vec::with_capacity(total_size);

        // Header
        out.extend_from_slice(&((total_size - 4) as i32).to_le_bytes());
        out.extend_from_slice(&(self.xsiz as i32).to_le_bytes());
        out.extend_from_slice(&(self.ysiz as i32).to_le_bytes());
        out.extend_from_slice(&(self.zsiz as i32).to_le_bytes());
        out.extend_from_slice(&((self.pivot.x * 256.0) as i32).to_le_bytes());
        out.extend_from_slice(&((self.pivot.y * 256.0) as i32).to_le_bytes());
        out.extend_from_slice(&((self.pivot.z * 256.0) as i32).to_le_bytes());

        // xoffsets (offsets to the start of each plane in xyoffset table)
        for i in 0..=self.xsiz {
            let off = i * (self.ysiz + 1) * 2;
            out.extend_from_slice(&(off as u32).to_le_bytes());
        }

        // xyoffsets
        for off in xyoffsets {
            out.extend_from_slice(&off.to_le_bytes());
        }

        // slabs
        out.extend_from_slice(&slab_bytes);

        out
    }

    /// Creates a synthetic procedural box model of size (xsiz, ysiz, zsiz) and color for testing/props.
    pub fn create_test_box(xsiz: usize, ysiz: usize, zsiz: usize, color: u8) -> Self {
        let mut model = Self::new(
            xsiz,
            ysiz,
            zsiz,
            Vec3::new(xsiz as f32 * 0.5, ysiz as f32 * 0.5, 0.0),
        );
        for x in 0..xsiz {
            for y in 0..ysiz {
                for z in 0..zsiz {
                    model.set_voxel(x, y, z, Some(color));
                }
            }
        }
        model
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_kvx_empty_and_get_set() {
        let mut model = KvxModel::new(4, 4, 4, Vec3::new(2.0, 2.0, 0.0));
        assert_eq!(model.get_voxel(0, 0, 0), None);
        assert_eq!(model.solid_count(), 0);

        model.set_voxel(1, 2, 3, Some(42));
        assert_eq!(model.get_voxel(1, 2, 3), Some(42));
        assert_eq!(model.solid_count(), 1);

        // Out of bounds check
        assert_eq!(model.get_voxel(10, 10, 10), None);
        model.set_voxel(10, 10, 10, Some(99));
        assert_eq!(model.solid_count(), 1);
    }

    #[test]
    fn test_kvx_encode_and_parse_roundtrip() {
        let mut original = KvxModel::new(8, 8, 8, Vec3::new(4.0, 4.0, 1.0));
        // Add voxels in multiple columns and slabs
        original.set_voxel(2, 2, 1, Some(10));
        original.set_voxel(2, 2, 2, Some(11));
        original.set_voxel(2, 2, 3, Some(12));
        original.set_voxel(5, 6, 0, Some(20));
        original.set_voxel(5, 6, 4, Some(21));

        let encoded = original.encode();
        assert!(encoded.len() > 28);

        let parsed = KvxModel::parse(&encoded).expect("Failed to parse encoded KVX");
        assert_eq!(parsed.xsiz, 8);
        assert_eq!(parsed.ysiz, 8);
        assert_eq!(parsed.zsiz, 8);
        assert!((parsed.pivot.x - 4.0).abs() < 0.01);
        assert!((parsed.pivot.y - 4.0).abs() < 0.01);
        assert!((parsed.pivot.z - 1.0).abs() < 0.01);

        assert_eq!(parsed.get_voxel(2, 2, 1), Some(10));
        assert_eq!(parsed.get_voxel(2, 2, 2), Some(11));
        assert_eq!(parsed.get_voxel(2, 2, 3), Some(12));
        assert_eq!(parsed.get_voxel(5, 6, 0), Some(20));
        assert_eq!(parsed.get_voxel(5, 6, 4), Some(21));
        assert_eq!(parsed.solid_count(), 5);
    }

    #[test]
    fn test_kvx_malformed_and_truncated_data() {
        // Less than header length
        assert_eq!(KvxModel::parse(&[0u8; 10]), Err(KvxError::UnexpectedEof));

        // Invalid dimensions (0 size)
        let mut zero_dim = vec![0u8; 32];
        zero_dim[0..4].copy_from_slice(&28i32.to_le_bytes());
        zero_dim[4..8].copy_from_slice(&0i32.to_le_bytes()); // xsiz = 0
        assert_eq!(KvxModel::parse(&zero_dim), Err(KvxError::InvalidDimensions));

        // Negative dimensions
        let mut neg_dim = vec![0u8; 32];
        neg_dim[4..8].copy_from_slice(&(-5i32).to_le_bytes());
        assert_eq!(KvxModel::parse(&neg_dim), Err(KvxError::InvalidDimensions));
    }

    #[test]
    fn test_kvx_test_box_procedural_generation() {
        let box_model = KvxModel::create_test_box(3, 4, 5, 17);
        assert_eq!(box_model.xsiz, 3);
        assert_eq!(box_model.ysiz, 4);
        assert_eq!(box_model.zsiz, 5);
        assert_eq!(box_model.solid_count(), 3 * 4 * 5);
        assert_eq!(box_model.get_voxel(2, 3, 4), Some(17));
    }
}
