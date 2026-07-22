use std::{env, process::Command};

#[cfg(windows)]
const NPX: &str = "npx.cmd";
#[cfg(not(windows))]
const NPX: &str = "npx";

fn main() {
    let public = env::current_dir().unwrap().join("public");
    let assets = env::current_dir().unwrap().join("assets");
    let src = env::current_dir().unwrap().join("src");
    let tailwind_in = assets.join("tailwind.css");
    let tailwind_out = public.join("tailwind.css");

    println!(
        "cargo:rustc-env=TAILWIND_CSS={}",
        tailwind_out.to_str().unwrap()
    );
    println!("cargo:rerun-if-changed={}", assets.to_str().unwrap());
    // Tailwind v4 scans `src` for class usage, so the output above is stale for any class added
    // or changed there until this build script reruns too - without this, cargo only reruns it
    // when `assets/tailwind.css` itself changes, silently leaving newly-used classes (e.g. a grid
    // layout added in a new component) without a generated rule.
    println!("cargo:rerun-if-changed={}", src.to_str().unwrap());

    Command::new(NPX)
        .arg("@tailwindcss/cli")
        .arg("-i")
        .arg(tailwind_in.to_str().unwrap())
        .arg("-o")
        .arg(tailwind_out.to_str().unwrap())
        .output()
        .expect("failed to build tailwindcss");
}
