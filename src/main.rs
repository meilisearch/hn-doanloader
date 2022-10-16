use futures::{stream, StreamExt};
use tokio;

const CONCURRENT_REQUESTS: usize = 20;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let max_item = reqwest::get("https://hacker-news.firebaseio.com/v0/maxitem.json")
        .await?
        .json::<u64>()
        .await?;

    let documents = stream::iter(0..max_item)
        .map(|i| async move {
            let mut count = 0;
            let resp = loop {
                let data = reqwest::get(format!(
                    "https://hacker-news.firebaseio.com/v0/item/{}.json",
                    i
                ))
                .await?;
                match data.status() {
                    reqwest::StatusCode::OK => {
                        if count > 0 {
                            eprintln!("count: {}", count);
                        }
                        break data;
                    }
                    code => {
                        if count >= 10 {
                            eprintln!("break.. {}", code);
                            break data;
                        }
                        count = count + 1;
                        continue;
                    }
                }
            };
            resp.text().await
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    documents
        .for_each(|b| async {
            match b {
                Ok(b) => println!("{}", b),
                Err(e) => eprintln!("Got a tokio::JoinError: {}", e),
            }
        })
        .await;
    Ok(())
}
