use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use headless_chrome::{Browser, LaunchOptions};
use serde_json;
use urlencoding::encode;
use scraper::{ElementRef, Html, Selector};

#[derive(Deserialize, Debug, PartialEq, Serialize)]
struct Styles {
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
struct Game {
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
    fn new(title: String, hltb_id: u32, main_story: Styles, main_extra: Styles, completionist: Styles, all_styles: Styles) -> Game {
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
    let title = document.select(&title_selector).next().unwrap().inner_html().trim().to_string().replace("<!-- -->", "");
    let table_selector = Selector::parse("#__next > div > main > div:nth-child(2) > div > div[class*='content_75_static'] > div.in.scrollable.scroll_blue.shadow_box.back_primary > table[class*='GameTimeTable_game_main_table']").unwrap();
    let table = document.select(&table_selector).next().unwrap();
    let tr_selector = Selector::parse("tbody > tr").unwrap();
    let mut rows = table.select(&tr_selector);
    let main_story = parse_row(rows.next().unwrap());
    let main_extra = parse_row(rows.next().unwrap());
    let completionist = parse_row(rows.next().unwrap());
    let all_styles = parse_row(rows.next().unwrap());
    Ok(Game::new(title,hltb_id, main_story, main_extra, completionist, all_styles))
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

fn to_json(game: Game) -> String {
    serde_json::to_string(&game).unwrap()
}

/// Searches for a game by name
///
/// # Arguments
///
/// * `name`:  &str - The name of the game to search for
///
/// returns: Result<String, Box<dyn Error, Global>>
pub async fn search_by_name(name: &str) -> Result<String, Box<dyn Error>> {
    let hltb_id = search_search_page_for(name).await.unwrap();
    let game = search_details_page_for(hltb_id).await.unwrap();
    Ok(to_json(game))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_search_search_page_for() {
        assert_eq!(search_search_page_for("Cyberpunk 2077").await.unwrap(), 2127);
    }

    #[tokio::test]
    async fn test_search_details_page_for() {
        let game = search_details_page_for(2127).await.unwrap();
        assert_eq!(game.hltb_id, 2127);
        assert_eq!(game.main_story, Styles::new(
            convert_hours_minutes_to_sec("26h 21m"),
            convert_hours_minutes_to_sec("24h 42m"),
            convert_hours_minutes_to_sec("17h 45m"),
            convert_hours_minutes_to_sec("42h 45m")
        ));
        assert_eq!(game.main_extra, Styles::new(
            convert_hours_minutes_to_sec("64h 52m"),
            convert_hours_minutes_to_sec("60h 0m"),
            convert_hours_minutes_to_sec("37h 31m"),
            convert_hours_minutes_to_sec("175h 13m")
        ));
        assert_eq!(game.completionist, Styles::new(
            convert_hours_minutes_to_sec("110h 58m"),
            convert_hours_minutes_to_sec("100h 35m"),
            convert_hours_minutes_to_sec("75h 53m"),
            convert_hours_minutes_to_sec("306h 23m")
        ));
        assert_eq!(game.all_styles, Styles::new(
            convert_hours_minutes_to_sec("68h 23m"),
            convert_hours_minutes_to_sec("60h 0m"),
            convert_hours_minutes_to_sec("37h 5m"),
            convert_hours_minutes_to_sec("285h 35m")
        ));
        assert_eq!(game.title, "Cyberpunk 2077");
    }

    #[tokio::test]
    async fn test_search_by_name() {
        let game = search_by_name("Cyberpunk 2077").await.unwrap();
        assert_eq!(game,  "{\"hltb_id\":2127,\"title\":\"Cyberpunk 2077\",\"main_story\":{\"average\":94860.0,\"median\":88920.0,\"rushed\":63900.0,\"leisure\":153900.0},\"main_extra\":{\"average\":233520.0,\"median\":216000.0,\"rushed\":135060.0,\"leisure\":630780.0},\"completionist\":{\"average\":399480.0,\"median\":362100.0,\"rushed\":273180.0,\"leisure\":1102980.0},\"all_styles\":{\"average\":246180.0,\"median\":216000.0,\"rushed\":133500.0,\"leisure\":1028100.0}}");
    }
}