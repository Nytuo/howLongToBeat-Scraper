use headless_chrome::{Browser, LaunchOptions};
use scraper::{ElementRef, Html, Selector};
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use urlencoding::encode;

#[derive(Deserialize, Debug, PartialEq, Serialize)]
pub struct Styles {
    average: f32,
    median: f32,
    rushed: f32,
    leisure: f32,
}

impl Styles {
    /// Creates a new Styles struct
    ///
    /// # Arguments
    ///
    /// * `average`:  f32 - The average time it takes to complete the game
    /// * `median`:  f32 - The median time it takes to complete the game
    /// * `rushed`:  f32 - The rushed time it takes to complete the game
    /// * `leisure`:  f32 - The leisure time it takes to complete the game
    ///
    /// returns: Styles
    fn new(average: f32, median: f32, rushed: f32, leisure: f32) -> Styles {
        Styles {
            average,
            median,
            rushed,
            leisure,
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
pub struct Game {
    hltb_id: u32,
    title: String,
    main_story: Styles,
    main_extra: Styles,
    completionist: Styles,
    all_styles: Styles,
}

impl Game {
    /// Creates a new Game struct
    ///
    /// # Arguments
    ///
    /// * `title`:  String - The title of the game
    /// * `hltb_id`:  u32 - The ID of the game on How Long to Beat
    /// * `main_story`:  Styles - The time it takes to complete the main story
    /// * `main_extra`:  Styles - The time it takes to complete the main story and extras
    /// * `completionist`:  Styles - The time it takes to complete the game 100%
    /// * `all_styles`:  Styles - The time it takes to complete the game in all styles
    ///
    /// returns: Game
    fn new(
        title: String,
        hltb_id: u32,
        main_story: Styles,
        main_extra: Styles,
        completionist: Styles,
        all_styles: Styles,
    ) -> Game {
        Game {
            hltb_id,
            title,
            main_story,
            main_extra,
            completionist,
            all_styles,
        }
    }
}

const BASE_URL: &str = "https://howlongtobeat.com/";

/// Searches the search page for a game
///
/// # Arguments
///
/// * `name`:  &str - The name of the game to search for
///
/// returns: Result<u32, Box<dyn Error, Global>>
async fn search_search_page_for(name: &str) -> Result<u32, Box<dyn Error>> {
    let url = BASE_URL.to_owned() + "?q=" + &encode(name);
    let launch_options = LaunchOptions {
        headless: true,
        ..Default::default()
    };
    let browser = Browser::new(launch_options)?;
    let tab = browser.new_tab()?;
    tab.set_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36", None, None)?;
    tab.navigate_to(&url)?;
    tab.wait_until_navigated()?;
    tab.wait_for_element("#search-results-header > ul > li:nth-child(1) > div > div[class*='GameCard_search_list_image'] > a")?;

    let content = tab.get_content()?;
    let document = Html::parse_document(&content);
    let selector = Selector::parse("#search-results-header > ul > li:nth-child(1) > div > div[class*='GameCard_search_list_image'] > a").unwrap();

    for element in document.select(&selector) {
        if let Some(link) = element.value().attr("href") {
            let id = link.split("/").last().unwrap().parse::<u32>().unwrap();
            return Ok(id);
        }
    }
    Err("Element not found".into())
}

/// Searches for the details page of a game
///
/// # Arguments
///
/// * `hltb_id`:  u32 - The ID of the game on How Long to Beat
///
/// returns: Result<Game, Box<dyn Error, Global>>
async fn search_details_page_for(hltb_id: u32) -> Result<Game, Box<dyn Error>> {
    let url = BASE_URL.to_owned() + "game/" + hltb_id.to_string().as_str();
    let launch_options = LaunchOptions {
        headless: true,
        ..Default::default()
    };
    let browser = Browser::new(launch_options)?;
    let tab = browser.new_tab()?;
    tab.set_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36", None, None)?;
    tab.navigate_to(&url)?;
    tab.wait_until_navigated()?;

    tab.wait_for_element("#__next > div > main > div:nth-child(2) > div > div[class*='content_75_static'] > div.in.scrollable.scroll_blue.shadow_box.back_primary > table[class*='GameTimeTable_game_main_table']")?;

    let content = tab.get_content()?;
    let document = Html::parse_document(&content);
    let title_selector = Selector::parse("#__next > div > main > div:nth-child(1) > div > div > div > div.GameHeader_profile_header__q_PID.shadow_text").unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .unwrap()
        .inner_html()
        .trim()
        .to_string()
        .replace("<!-- -->", "");
    let table_selector = Selector::parse("#__next > div > main > div:nth-child(2) > div > div[class*='content_75_static'] > div.in.scrollable.scroll_blue.shadow_box.back_primary > table[class*='GameTimeTable_game_main_table']").unwrap();
    let table = document.select(&table_selector).next().unwrap();
    let tr_selector = Selector::parse("tbody > tr").unwrap();
    let mut rows = table.select(&tr_selector);
    let main_story = parse_row(rows.next().unwrap());
    let main_extra = parse_row(rows.next().unwrap());
    let completionist = parse_row(rows.next().unwrap());
    let all_styles = parse_row(rows.next().unwrap());
    Ok(Game::new(
        title,
        hltb_id,
        main_story,
        main_extra,
        completionist,
        all_styles,
    ))
}

/// Parses a row of a table
///
/// # Arguments
///
/// * `row`:  ElementRef - The row to parse
///
/// returns: Styles
fn parse_row(row: ElementRef) -> Styles {
    let selector = Selector::parse("td").unwrap();
    let mut cells = row.select(&selector);
    cells.next();
    cells.next();
    let average = convert_hours_minutes_to_sec(cells.next().unwrap().inner_html().as_str());
    let median = convert_hours_minutes_to_sec(cells.next().unwrap().inner_html().as_str());
    let rushed = convert_hours_minutes_to_sec(cells.next().unwrap().inner_html().as_str());
    let leisure = convert_hours_minutes_to_sec(cells.next().unwrap().inner_html().as_str());
    Styles::new(average, median, rushed, leisure)
}

/// Converts a string of hours and minutes to seconds
///
/// # Arguments
///
/// * `text`:  &str - The text to convert to seconds (e.g. "26h 21m")
///
/// returns: f32
fn convert_hours_minutes_to_sec(text: &str) -> f32 {
    let parts = text.split(" ");
    let mut total = 0.0;
    for part in parts {
        if part.contains("h") {
            total += part.replace("h", "").parse::<f32>().unwrap() * 3600.0;
        } else {
            total += part.replace("m", "").parse::<f32>().unwrap() * 60.0;
        }
    }
    total
}

/// Searches for a game by name
///
/// # Arguments
///
/// * `name`:  &str - The name of the game to search for
///
/// returns: Result<String, Box<dyn Error, Global>>
pub async fn search_by_name(name: &str) -> Result<Game, Box<dyn Error>> {
    let hltb_id = search_search_page_for(name).await.unwrap();
    let game = search_details_page_for(hltb_id).await.unwrap();
    Ok(game)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_search_search_page_for() {
        assert_eq!(search_search_page_for("Metal Gear").await.unwrap(), 5900);
    }

    #[tokio::test]
    async fn test_search_details_page_for() {
        let game = search_details_page_for(5900).await.unwrap();
        assert_eq!(game.hltb_id, 5900);
        assert_eq!(
            game.main_story,
            Styles::new(
                convert_hours_minutes_to_sec("4h 8m"),
                convert_hours_minutes_to_sec("4h"),
                convert_hours_minutes_to_sec("2h 45m"),
                convert_hours_minutes_to_sec("7h 12m")
            )
        );
        assert_eq!(
            game.main_extra,
            Styles::new(
                convert_hours_minutes_to_sec("5h"),
                convert_hours_minutes_to_sec("4h 55m"),
                convert_hours_minutes_to_sec("3h 34m"),
                convert_hours_minutes_to_sec("7h 31m")
            )
        );
        assert_eq!(
            game.completionist,
            Styles::new(
                convert_hours_minutes_to_sec("5h 25m"),
                convert_hours_minutes_to_sec("5h"),
                convert_hours_minutes_to_sec("4h 5m"),
                convert_hours_minutes_to_sec("10h 36m")
            )
        );
        assert_eq!(
            game.all_styles,
            Styles::new(
                convert_hours_minutes_to_sec("4h 31m"),
                convert_hours_minutes_to_sec("4h"),
                convert_hours_minutes_to_sec("2h 51m"),
                convert_hours_minutes_to_sec("10h 7m")
            )
        );
        assert_eq!(game.title, "Metal Gear");
    }

    #[tokio::test]
    async fn test_search_by_name() {
        let game = search_by_name("Metal Gear").await.unwrap();
        let expected = Game::new(
            "Metal Gear".to_string(),
            5900,
            Styles::new(
                convert_hours_minutes_to_sec("4h 8m"),
                convert_hours_minutes_to_sec("4h"),
                convert_hours_minutes_to_sec("2h 45m"),
                convert_hours_minutes_to_sec("7h 12m"),
            ),
            Styles::new(
                convert_hours_minutes_to_sec("5h"),
                convert_hours_minutes_to_sec("4h 55m"),
                convert_hours_minutes_to_sec("3h 34m"),
                convert_hours_minutes_to_sec("7h 31m"),
            ),
            Styles::new(
                convert_hours_minutes_to_sec("5h 25m"),
                convert_hours_minutes_to_sec("5h"),
                convert_hours_minutes_to_sec("4h 5m"),
                convert_hours_minutes_to_sec("10h 36m"),
            ),
            Styles::new(
                convert_hours_minutes_to_sec("4h 31m"),
                convert_hours_minutes_to_sec("4h"),
                convert_hours_minutes_to_sec("2h 51m"),
                convert_hours_minutes_to_sec("10h 7m"),
            ),
        );
        assert_eq!(game, expected);
    }
}
