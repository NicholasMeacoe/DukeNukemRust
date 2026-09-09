fn main() {
    let mut grp = dukenukemrust::grp::Grp::open("../dukenukem3d/duke3d.grp").unwrap();
    let map_data = grp.read_file("E1L1.MAP").unwrap();
    let map = dukenukemrust::map::Map::from_bytes(&map_data).unwrap();
    let sec = &map.sectors[306];
    println!("SECTOR 306 floorz: {} ceilingz: {}", sec.floorz, sec.ceilingz);
    for i in 0..sec.wallnum {
        let w = &map.walls[(sec.wallptr + i) as usize];
        println!("Wall {}: picnum={} overpicnum={} nextsector={} cstat={}", i, w.picnum, w.overpicnum, w.nextsector, w.cstat);
    }
}
