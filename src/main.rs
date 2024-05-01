use chrono::{Datelike, TimeZone, Utc};
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::{fs, io::Cursor, time::{Duration, Instant}};
use xml::{reader::XmlEvent, EventReader};

fn main() {
    let start = Instant::now();
    let now = Utc::now();
    let current_year = now.year() as usize;
    let current_month = now.month() as usize;
    // this is a very silly way of doing things but it works
    let mut counter = vec![[(0, 0); 12]; current_year + 1];
    let client = Client::builder().timeout(Duration::from_secs(60)).build().unwrap();
    // jvs
    println!("jvs");
    let stuff = client.get(format!(
        "https://jbovlaste.lojban.org/recent.html?days={:?}",
        (now - Utc::with_ymd_and_hms(&Utc{}, 2003, 1, 1, 0, 0, 0).unwrap()).num_days()
    )).send().unwrap().text().unwrap();
    let updates = Html::parse_document(&stuff);
    let sel = Selector::parse(r#"td[width="80%"]"#).unwrap();
    let updates = updates.select(&sel).next().unwrap().text();
    let updates = updates.clone().skip(3).zip(updates).filter(|(a, b)| [a, b].iter().any(|x| x.contains("valsi originally entered"))).map(|x| x.0).collect::<Vec<_>>();
    // find the ghosts
    let xml = client.get("https://jbovlaste.lojban.org/export/xml-export.html?lang=en&positive_scores_only=0&bot_key=z2BsnKYJhAB0VNsl").send().unwrap().bytes().unwrap();
    let mut reader = EventReader::new(Cursor::new(xml));
    let mut xml_words = vec![];
    loop {
        match reader.next().unwrap() {
            XmlEvent::EndDocument{..} => {
                break;
            }
            XmlEvent::StartElement{name, attributes, ..} => {
                if name.local_name == "valsi" {
                    xml_words.push(attributes.iter().find(|&x| x.name.local_name == "word").unwrap().value.clone());
                }
            }
            _ => ()
        }
    }
    let mut ghosts = vec![];
    for t in updates {
        if t.contains("valsi originally entered") {
            let date = &t.replace('\n', "")[3..11];
            let y = date[4..8].parse::<usize>().unwrap();
            let m = match &date[0..3] {
                "Jan" => 0,
                "Feb" => 1,
                "Mar" => 2,
                "Apr" => 3,
                "May" => 4,
                "Jun" => 5,
                "Jul" => 6,
                "Aug" => 7,
                "Sep" => 8,
                "Oct" => 9,
                "Nov" => 10,
                "Dec" => 11,
                _ => panic!("wtf kinda month is `{}`?", &date[0..3])
            };
            counter[y][m].0 += 1;
        } else if !xml_words.contains(&t.to_string()) {
            ghosts.push(t);
        }
    }
    ghosts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    // toadua
    println!("toadua");
    let stuff = client.post("https://toadua.uakci.pl/api").body(r#"{"action": "search", "query": ["scope", "en"]}"#).send().unwrap();
    let stuff = serde_json::from_reader::<_, Toadua>(stuff).unwrap();
    for t in stuff.results {
        let the = t.date.split('-').collect::<Vec<_>>();
        let y = the[0].parse::<usize>().unwrap();
        let m = the[1].parse::<usize>().unwrap() - 1;
        counter[y][m].1 += 1;
    }
    let mut out = String::new();
    let (mut jbo_t, mut toaq_t) = (0, 0);
    for (y, _) in counter.iter().enumerate() {
        for (m, (jbo, toaq)) in counter[y].iter().enumerate() {
            let m = m + 1;
            jbo_t += jbo;
            toaq_t += toaq;
            if y >= 2003 && !(y == current_year && m > current_month) {
                out += &format!("{y}-{m:02}\t{jbo_t}\t{toaq_t}\r\n");
            }
        }
    }
    fs::write("out.tsv", out).unwrap();
    let out = ghosts.join("\r\n");
    fs::write("jbosts.txt", out).unwrap();
    println!("{:?}", Instant::now() - start);
}

#[derive(Deserialize)]
struct Toadua {
    results: Vec<Toa>
}

#[derive(Deserialize)]
struct Toa {
    date: String
}
