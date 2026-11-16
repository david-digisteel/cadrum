//! An exception OpenCascade raises comes back to Rust: as `Error::Kernel` from a `try_`
//! call, as a panic naming the call from its infallible form. Neither ends the process.

use cadrum::{Error, Solid};
use glam::DVec3;

fn kernel_message<T>(result: Result<T, Error>) -> String {
	match result {
		Err(Error::Kernel(message)) => message,
		Err(other) => panic!("expected Error::Kernel, got {other}"),
		Ok(_) => panic!("expected Error::Kernel, got a value"),
	}
}

fn panic_message(f: impl FnOnce() + std::panic::UnwindSafe) -> String {
	let payload = std::panic::catch_unwind(f).expect_err("expected a panic");
	payload.downcast_ref::<String>().cloned().or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default()
}

#[test]
fn a_degenerate_primitive_is_an_error() {
	// BRepPrim_GWedge throws Standard_DomainError for a box with no extent.
	let flat = kernel_message(Solid::try_cube(DVec3::ZERO, DVec3::new(1.0, 1.0, 0.0)));
	assert!(flat.starts_with("cube: "), "{flat}");
	assert!(flat.contains("Standard_DomainError"), "{flat}");
	// gp_Dir throws Standard_ConstructionError for a zero axis.
	let axis = kernel_message(Solid::try_cylinder(1.0, DVec3::ZERO));
	assert!(axis.contains("Standard_ConstructionError"), "{axis}");
	assert!(kernel_message(Solid::try_cone(1.0, 0.5, DVec3::ZERO)).contains("Standard_ConstructionError"));
	assert!(kernel_message(Solid::try_torus(2.0, 1.0, DVec3::ZERO)).contains("Standard_ConstructionError"));
	assert!(kernel_message(Solid::try_half_space(DVec3::ZERO, DVec3::ZERO)).contains("Standard_ConstructionError"));
}

#[test]
fn the_infallible_form_panics_with_the_kernels_message() {
	let message = panic_message(|| {
		Solid::cube(DVec3::ZERO, DVec3::new(1.0, 1.0, 0.0));
	});
	assert!(message.starts_with("Solid::cube: Kernel failed: cube: "), "{message}");
	assert!(message.contains("Standard_DomainError"), "{message}");
	let message = panic_message(|| {
		Solid::cylinder(1.0, DVec3::ZERO);
	});
	assert!(message.contains("Standard_ConstructionError"), "{message}");
}

#[test]
fn a_valid_call_through_the_try_forms_matches_the_infallible_one() {
	let cube = Solid::try_cube(DVec3::ZERO, DVec3::splat(2.0)).unwrap().try_translate(DVec3::new(1.0, 0.0, 0.0)).unwrap();
	assert!((cube.volume() - 8.0).abs() < 1e-9);
	assert!((cube.bounding_box()[0] - DVec3::new(1.0, 0.0, 0.0)).length() < 1e-6);
	let faces: Vec<_> = cube.try_iter_face().unwrap().collect();
	assert_eq!(faces.len(), 6);
	let area: f64 = faces.iter().map(|f| f.try_area().unwrap()).sum();
	assert!((area - 24.0).abs() < 1e-9);
	assert_eq!(cube.try_iter_edge().unwrap().count(), 12);
	assert_eq!(cube.iter_face().count(), 6);
	let cylinder = Solid::try_cylinder(1.0, DVec3::new(0.0, 0.0, 3.0)).unwrap();
	assert!((cylinder.volume() - 3.0 * std::f64::consts::PI).abs() < 1e-9);
}

// A check inside OpenCascade's compiled code (gp_Trsf::SetScale in TKMath), not in a header
// the bridge compiles: it raises only when OCCT is built with BUILD_RELEASE_DISABLE_EXCEPTIONS
// off, as the `source` feature builds it. The prebuilt kernels predate that.
#[test]
#[cfg_attr(not(feature = "source"), ignore = "needs an OCCT built with its checks compiled in (the source feature)")]
fn occt_checks_are_compiled_in() {
	let message = panic_message(|| {
		Solid::cube(DVec3::ZERO, DVec3::ONE).scale(DVec3::ZERO, 0.0);
	});
	assert!(message.starts_with("Solid::scale: Standard_ConstructionError"), "{message}");
}
