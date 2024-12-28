# howlongtobeat-scraper

Simple API for [HowLongToBeat](https://howlongtobeat.com).

## Description

`howlongtobeat-scraper` is a Rust library that provides a simple API to interact with the HowLongToBeat website. It allows you to scrape game information such as playtime estimates.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
howlongtobeat-scraper = "0.1.0"
```

## Usage

Here is a basic example of how to use the library:

```rust
use howlongtobeat_scraper::HowLongToBeatScraper;

fn main() {
    let game_info = search_by_name("The Legend of Zelda: Breath of the Wild").unwrap();
    println!("{:?}", game_info);
}
```

## Output

The output will be a JSON string containing the game information. For example:

```json
{
    "hltb_id": 2127,
    "title": "Cyberpunk 2077",
    "main_story": {
        "average": 94860.0,
        "median": 88920.0,
        "rushed": 63900.0,
        "leisure": 153900.0
    },
    "main_extra": {
        "average": 233520.0,
        "median": 216000.0,
        "rushed": 135060.0,
        "leisure": 630780.0
    },
    "completionist": {
        "average": 399480.0,
        "median": 362100.0,
        "rushed": 273180.0,
        "leisure": 1102980.0
    },
    "all_styles": {
        "average": 246180.0,
        "median": 216000.0,
        "rushed": 133500.0,
        "leisure": 1028100.0
    }
}
```

## Features

- Scrape game information from HowLongToBeat
- Retrieve playtime estimates for different game categories

## License

This project is licensed under the MIT License. See the `LICENSE` file for more details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on [GitHub](https://github.com/nytuo/howlongtobeat-scraper).

## Author

Arnaud BEUX - [GitHub](https://github.com/nytuo) - nytuo.yt@gmail.com