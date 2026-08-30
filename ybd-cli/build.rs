fn main() {
        let output = std::process::Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .output()
                .unwrap();
        let hash = String::from_utf8(output.stdout).unwrap();
        let hash = hash.trim();
        println!("cargo:rustc-env=GIT_HASH={}", hash);
}
