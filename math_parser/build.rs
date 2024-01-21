use std::fs;

fn main() {
    let version_str = fs::read_to_string(r"D:\Documents\Programming\Rust\math_parser-rs\data\version.no").expect("Cannot find the version.no file.");
    let (major, minor, build) = create_or_update(version_str.as_str());

    println!("cargo:rustc-env=MATH_MAJOR={}", major);
    println!("cargo:rustc-env=MATH_MINOR={}", minor);
    println!("cargo:rustc-env=MATH_BUILD={}", build);
}


fn create_or_update(version_str: &str) -> (u32, u32, u32) {
    let parts: Vec<_> = version_str.split(".").collect();
    if parts.len() != 3 {
        panic!("version file does not contain a value in the format 1.2.3");
    }
    let Ok(major) = parts[0].parse::<u32>() else {
        panic!("major version number is not an int: `{}`", parts[0]);
    };
    let Ok(minor) = parts[1].parse::<u32>() else {
        panic!("minor version number is not an int: `{}`", parts[1]);
    };
    let Ok(build) = parts[2].parse::<u32>() else {
        panic!("build version number is not an int: `{}`", parts[2]);
    };
    (major, minor, build)
}
