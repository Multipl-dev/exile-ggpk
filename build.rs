// use std::env;
use std::path::PathBuf;

fn main() {
    let ooz_path = PathBuf::from(r"s:\_projects_\_poe2_\ooz");

    if !ooz_path.exists() {
        println!("cargo:warning=ooz directory not found at {:?}", ooz_path);
        return;
    }

    println!("cargo:rerun-if-changed={}", ooz_path.display());

    let mut build = cc::Build::new();
    
    build
        .cpp(true)
        .std("c++17")
        .define("BUN_BUILD_DLL", "1")
        .define("OOZ_BUILD_DLL", "1") // Prevents kraken.cpp from defining main()
        .flag("/EHsc")
        .include(&ooz_path)
        .include(ooz_path.join("simde"));

    let files = vec![
        "bun.cpp",
        "kraken.cpp",
        "bitknit.cpp",
        "lzna.cpp",
        "compr_entropy.cpp",
        "compr_kraken.cpp",
        "compr_leviathan.cpp",
        "compr_match_finder.cpp",
        "compr_mermaid.cpp",
        "compr_multiarray.cpp",
        "compr_tans.cpp",
        "compress.cpp", 
        "fnv.cpp", 
        "murmur.cpp", 
        "utf.cpp", 
        "util.cpp",
    ];

    for file in files {
        let p = ooz_path.join(file);
        if p.exists() {
             build.file(p);
        } else {
            println!("cargo:warning=File not found: {:?}", p);
        }
    }

    build.compile("ooz");

    println!("cargo:rustc-link-lib=static=ooz");
}
