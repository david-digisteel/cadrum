//! The ASCII `BRepTools` reader, `Solid::fix`, and `Mesh::unmeshed_share`.
//!
//! The sample is a 10 x 20 x 30 box with a 6 mm hole through its 10 mm thickness, written
//! in OCCT's text flavour by another OCCT build. Its volume and area have closed forms,
//! so the tests check that what was read is the part the file describes.

use cadrum::{Solid, Tessellation};
use std::f64::consts::PI;

const SAMPLE: &[u8] = include_bytes!("data/box_with_hole.text.brep");
const VOLUME: f64 = 10.0 * 20.0 * 30.0 - PI * 9.0 * 10.0;
const AREA: f64 = 2.0 * (10.0 * 20.0 + 10.0 * 30.0 + 20.0 * 30.0) - 2.0 * PI * 9.0 + 2.0 * PI * 3.0 * 10.0;

fn sample() -> Vec<Solid> {
	Solid::read_brep_text(&mut &SAMPLE[..]).expect("the text sample reads")
}

#[test]
fn text_brep_reads_the_part_it_describes() {
	let solids = sample();
	assert_eq!(solids.len(), 1, "one solid in the file");
	let solid = &solids[0];
	assert!((solid.volume() - VOLUME).abs() < 1e-6 * VOLUME, "volume {} against {VOLUME}", solid.volume());
	assert!((solid.area() - AREA).abs() < 1e-6 * AREA, "area {} against {AREA}", solid.area());
	// Six box faces and the bore's wall.
	assert_eq!(solid.iter_face().count(), 7);
}

#[test]
fn text_and_binary_flavours_carry_the_same_solid() {
	let from_text = sample();
	let mut binary = Vec::new();
	Solid::write_brep(&from_text, &mut binary).expect("binary write");
	let from_binary = Solid::read_brep(&mut &binary[..]).expect("binary read");
	assert_eq!(from_binary.len(), from_text.len());
	assert!((from_binary[0].volume() - from_text[0].volume()).abs() < 1e-9 * VOLUME);
	assert_eq!(from_binary[0].iter_face().count(), from_text[0].iter_face().count());
}

#[test]
fn each_reader_refuses_the_other_flavour() {
	// A file handed to the wrong reader must fail loudly, never come back as an empty part.
	assert!(Solid::read_brep(&mut &SAMPLE[..]).is_err(), "the binary reader took a text file");
	let mut binary = Vec::new();
	Solid::write_brep(&sample(), &mut binary).unwrap();
	assert!(Solid::read_brep_text(&mut &binary[..]).is_err(), "the text reader took a binary file");
	assert!(Solid::read_brep_text(&mut &b"not a shape at all"[..]).is_err());
}

#[test]
fn a_whole_mesh_reports_nothing_unmeshed() {
	let tess = Tessellation { deflection_linear: 0.05, relative_linear: false, ..Default::default() };
	let mesh = Solid::mesh(&sample(), tess).expect("mesh");
	assert!(!mesh.indices.is_empty());
	assert_eq!(mesh.unmeshed_share, 0.0, "every face of a sound part is triangulated");
	// The mesh covers the part: its area approaches the exact one from below on the bore.
	let area: f64 = mesh.indices.chunks_exact(3).map(|t| (mesh.vertices[t[1]] - mesh.vertices[t[0]]).cross(mesh.vertices[t[2]] - mesh.vertices[t[0]]).length() / 2.0).sum();
	assert!((area - AREA).abs() < 1e-2 * AREA, "mesh area {area} against {AREA}");
}

#[test]
fn fix_leaves_a_sound_part_as_it_was() {
	let before = sample();
	// No face of a sound part bounds the complement of its domain.
	assert!(before[0].iter_face().all(|f| f.area() > 0.0));
	let after = Solid::fix(&before).expect("fix");
	assert_eq!(after.len(), 1);
	assert!((after[0].volume() - VOLUME).abs() < 1e-6 * VOLUME);
	assert_eq!(after[0].iter_face().count(), 7);
}
