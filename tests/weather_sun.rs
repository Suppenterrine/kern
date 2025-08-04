use chrono::{DateTime, Utc};
use kern::core::{sun, weather};
use mockito::Matcher;

#[test]
fn test_fetch_current_weather_mock() {
    let mut server = mockito::Server::new();
    let m = server.mock("GET", "/v1/forecast")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("latitude".into(), "52.5".into()),
            Matcher::UrlEncoded("longitude".into(), "13.4".into()),
            Matcher::UrlEncoded("current_weather".into(), "true".into()),
        ]))
        .with_status(200)
        .with_body(r#"{ "current_weather": { "temperature": 20.0, "windspeed": 5.0, "winddirection": 90.0, "weathercode": 0, "time": "2025-08-04T12:00" } }"#)
        .create();

    let cw = weather::fetch_current_weather_from(52.5, 13.4, &server.url()).unwrap();
    assert_eq!(cw.temperature, 20.0);
    assert_eq!(cw.windspeed, 5.0);
    assert_eq!(cw.winddirection_deg, 90.0);
    assert_eq!(cw.weathercode, 0);
    assert_eq!(weather::deg_to_compass(cw.winddirection_deg), "E");
    m.assert();
}

#[test]
fn test_solar_position() {
    let dt = DateTime::parse_from_rfc3339("2025-08-04T12:00:00Z").unwrap().with_timezone(&Utc);
    let pos = sun::solar_position(52.5, 13.4, dt);
    assert!((pos.azimuth - 199.25).abs() < 0.5);
    assert!((pos.elevation - 53.39).abs() < 0.5);
}
