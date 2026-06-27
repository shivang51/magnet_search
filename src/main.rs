use std::io::Write;

use magneto::{Knaben, Magneto, OrderBy, PirateBay, SearchRequest};
use urlencoding::encode;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 || (args.len() > 1 && args[1] == "--help") {
        println!("Usage: magnet_search <query>");
        println!("Example: magnet_search \"ubuntu iso\"");
        println!("Args:");
        println!("\tquery: The search query to find torrents");
        println!("Options:");
        println!("\t--help: Show this help message");
        return;
    }

    let magneto =
        Magneto::with_providers(vec![Box::new(Knaben::new()), Box::new(PirateBay::new())]);

    let query = args[1].clone();

    let request = SearchRequest {
        query: query.as_str(),
        order_by: OrderBy::Seeders,
        categories: vec![],
        number_of_results: 10,
    };

    println!("Searching for torrents matching: \"{}\"...", query);

    let mut id = 0;
    match magneto.search(request).await {
        Ok(results) => {
            println!("Found {} results:", results.len());
            for torrent in results.iter() {
                println!(
                    "{} > Title: {} | Seeders: {} | Size: {}MB | Provider: {}",
                    id,
                    torrent.name,
                    torrent.seeders,
                    torrent.size_bytes / (1024 * 1024),
                    torrent.provider
                );
                id += 1;
            }

            print!("\nEnter the ID of the torrent you want to download: ");
            std::io::stdout().flush().expect("Failed to flush stdout");

            let mut id_input = String::new();
            std::io::stdin()
                .read_line(&mut id_input)
                .expect("Failed to read line");
            let id_input = id_input.trim();
            let sel_id: usize = match id_input.parse() {
                Ok(num) => num,
                Err(_) => {
                    eprintln!("Invalid ID");
                    return;
                }
            };

            if sel_id >= results.len() {
                eprintln!("ID out of range");
                return;
            }

            let selected_torrent = &results[sel_id];
            let link = add_trackers(&selected_torrent.magnet_link, &selected_torrent.name).await;
            println!("Selected torrent: {}", selected_torrent.name);

            match open::that_detached(&link) {
                Ok(_) => println!("Opening magnet link {}", link),
                Err(e) => eprintln!("Failed to open magnet link: {}", e),
            }
        }
        Err(e) => eprintln!("Error searching: {}", e),
    }
}

async fn fetch_latest_trackers() -> Vec<String> {
    const URL: &'static str =
        "https://raw.githubusercontent.com/ngosang/trackerslist/master/trackers_best.txt";

    let response = reqwest::get(URL)
        .await
        .expect("Failed to fetch trackers list");

    let body = response.text().await.expect("Failed to read response body");
    let trackers: Vec<String> = body
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| String::from(line.trim()))
        .collect();

    trackers
}

async fn add_trackers(raw_magnet: &str, display_name: &str) -> String {
    let stable_trackers = fetch_latest_trackers().await;

    let mut full_magnet = format!("{}&dn={}", raw_magnet, encode(display_name));

    for tracker in &stable_trackers {
        full_magnet.push_str(&format!("&tr={}", encode(tracker)));
    }

    full_magnet
}
