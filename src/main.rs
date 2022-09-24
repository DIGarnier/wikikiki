use std::{
    collections::{HashSet, VecDeque},
    time::Instant,
};

use rayon::prelude::{ParallelDrainRange, ParallelIterator};

fn extract_links(doc: &str) -> Vec<String> {
    let mut links = Vec::new();
    
    if doc.find("Wikimedia Error").is_some() {
        println!("nogood");
        return links;
    }
    
    let mut start = 0usize;
    while start < doc.len() {
        match doc[start..].find("<a href=\"/wiki/") {
            Some(x) => {
                let x = x + start + 10;
                let end = doc[x..].find("\"").unwrap() + x;
                links.push(doc[x + 5..end].to_owned());
                start = end;
            }
            None => break,
        }
    }

    links
}




#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();

    let mut urls_to_visit = VecDeque::from(["Canada".to_owned()]);
    let mut unicity_holder = HashSet::new();

    let mut total_visited = 0usize;
    let total_time = Instant::now();
    while !urls_to_visit.is_empty() {
        let started_at = Instant::now();
        let n = 1000.min(urls_to_visit.len());
        
        let a = urls_to_visit
            .par_drain(..n)
            .map(|url| {
                let client = &client;
                async move {
                    let doc = client
                        .clone()
                        .get(format!("https://en.wikipedia.org/wiki/{}", url.clone()))
                        .send()
                        .await
                        .unwrap()
                        .text()
                        .await
                        .unwrap();
                    extract_links(&doc)
                }
            })
            .collect::<Vec<_>>();

        let a = futures::future::join_all(a).await.into_iter().flatten().collect::<Vec<String>>();

        unicity_holder.extend(a.clone());
        urls_to_visit.extend(a);
        total_visited += n;
        println!(
            "total nb req: {} | total time: {} s | avgtime: {} ms | nb unique {}",
            total_visited,
            total_time.elapsed().as_secs(),
            started_at.elapsed().as_millis() as f64 / n as f64,
            unicity_holder.len()
        );
    }
}