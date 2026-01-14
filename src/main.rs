use crator::crate_data;

#[tokio::main]
async fn main() {
    let crate_name = "crator";
    let info = crate_data(crate_name).await.expect("Failed to get crate info");
    println!(
        "Latest: v{}, Downloads: {}, Total Downloads: {}, License: {}",
        info.latest, info.downloads, info.total_downloads, info.license
    );
}