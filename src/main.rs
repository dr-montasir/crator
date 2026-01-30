use crator::*;

fn main() {
    let crate_name = "cans";
    let start = Instant::now();

    // Work happens here...
    let info = block_on(crate_data(crate_name)).expect("Failed to get crate info");

    // ...then print the timing!
    println!("🦀 Fetching [{}] done in {:?}", crate_name, start.elapsed());

    println!("Latest:    v{}", info.latest);
    println!("Versions:  {}", info.versions);
    println!("Downloads: {}", info.downloads);
} 