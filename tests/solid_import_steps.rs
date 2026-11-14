//! The kernel-side steps of an import: `Solid::fuse`, `Solid::unify`, `Solid::properties`,
//! `Solid::remove_internal_edges`, and the corner a reversed face's triangles are listed from.

use cadrum::{DVec3, Solid, Tessellation};
use std::f64::consts::PI;

const BOX_WITH_HOLE: &[u8] = include_bytes!("data/box_with_hole.text.brep");

fn cube(x0: f64, x1: f64) -> Solid {
	Solid::cube(DVec3::new(x0, 0.0, 0.0), DVec3::new(x1, 10.0, 10.0))
}

#[test]
fn abutting_solids_fuse_to_one_and_separate_lumps_stay_apart() {
	// Two cubes sharing the face x = 10 are one piece of steel drawn as two.
	let touching = Solid::fuse(&[cube(0.0, 10.0), cube(10.0, 20.0)]).expect("fuse");
	assert_eq!(touching.len(), 1, "bodies that share a face weld into one");
	assert!((touching[0].volume() - 2000.0).abs() < 1e-6, "the weld holds both cubes: {}", touching[0].volume());

	// Two cubes a gap apart are two parts, and a union does not make them one.
	let apart = Solid::fuse(&[cube(0.0, 10.0), cube(15.0, 25.0)]).expect("fuse");
	assert_eq!(apart.len(), 2, "separate lumps come back as many as went in");
}

#[test]
fn unify_joins_the_faces_a_union_left_in_pieces() {
	let fused = Solid::fuse(&[cube(0.0, 10.0), cube(10.0, 20.0)]).expect("fuse");
	// The union keeps each cube's four side faces as its own: 4 + 4 sides, 2 ends.
	assert_eq!(fused[0].iter_face().count(), 10);
	let unified = Solid::unify(&fused, false).expect("unify");
	assert_eq!(unified.len(), 1);
	assert_eq!(unified[0].iter_face().count(), 6, "a 20 x 10 x 10 block has six faces");
	assert!((unified[0].volume() - 2000.0).abs() < 1e-6, "joining faces moves no steel");
}

#[test]
fn properties_measure_the_part_and_count_shared_edges_once() {
	let solids = Solid::read_brep_text(&mut &BOX_WITH_HOLE[..]).expect("the text sample reads");
	let p = Solid::properties(&solids, 1e-8).expect("properties");
	let volume = 10.0 * 20.0 * 30.0 - PI * 9.0 * 10.0;
	let area = 2.0 * (10.0 * 20.0 + 10.0 * 30.0 + 20.0 * 30.0) - 2.0 * PI * 9.0 + 2.0 * PI * 3.0 * 10.0;
	assert!((p.volume - volume).abs() < 1e-7 * volume, "volume {} against {volume}", p.volume);
	assert!((p.area - area).abs() < 1e-7 * area, "area {} against {area}", p.area);
	// Six box faces and the bore's wall; twelve box edges, the bore's two rims and its seam.
	assert_eq!(p.faces, 7);
	assert_eq!(p.edges, 15);

	// Over two solids the measures add.
	let two = [cube(0.0, 10.0), cube(15.0, 25.0)];
	let q = Solid::properties(&two, 1e-8).expect("properties");
	assert!((q.volume - 2000.0).abs() < 1e-6);
	assert!((q.area - 1200.0).abs() < 1e-6);
	assert_eq!((q.faces, q.edges), (12, 24));
}

#[test]
fn a_shape_without_internal_edges_is_left_as_it_is() {
	let solids = Solid::read_brep_text(&mut &BOX_WITH_HOLE[..]).expect("the text sample reads");
	let before = Solid::properties(&solids, 1e-8).expect("properties");
	let cleaned = Solid::remove_internal_edges(&solids).expect("remove internal edges");
	let after = Solid::properties(&cleaned, 1e-8).expect("properties");
	assert_eq!(cleaned.len(), solids.len());
	assert_eq!(before, after, "nothing internal to remove, so nothing moves");
}

#[test]
fn every_triangle_winds_outward_whichever_way_its_face_is_oriented() {
	// A box with a bore has faces of both orientations. Whatever corner a triangle is
	// listed from, its winding must agree with the outward normals at its corners.
	let solids = Solid::read_brep_text(&mut &BOX_WITH_HOLE[..]).expect("the text sample reads");
	let mesh = Solid::mesh(&solids, Tessellation { deflection_linear: 0.05, deflection_angular: 0.5, relative_linear: false }).expect("mesh");
	assert!(mesh.indices.len() >= 3 * 12);
	for t in mesh.indices.chunks(3) {
		let (a, b, c) = (mesh.vertices[t[0]], mesh.vertices[t[1]], mesh.vertices[t[2]]);
		let winding = (b - a).cross(c - a);
		let normal = mesh.normals[t[0]] + mesh.normals[t[1]] + mesh.normals[t[2]];
		assert!(winding.dot(normal) > 0.0, "a triangle winds against its normals");
	}
}
