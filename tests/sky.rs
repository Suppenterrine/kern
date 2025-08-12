use chrono::{DateTime, Utc};
use kern::core::{sky, sun, weather};
use mockito::Matcher;

#[test]
fn test_fetch_current_weather_mock() {
    let mut server = mockito::Server::new();
    let m = server
        .mock("GET", "/v1/forecast")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("latitude".into(), "52.5".into()),
            Matcher::UrlEncoded("longitude".into(), "13.4".into()),
            Matcher::UrlEncoded("current_weather".into(), "true".into()),
        ]))
        .with_status(200)
        .with_body(
            r#"{ "current_weather": { "temperature": 20.0, "windspeed": 5.0, "winddirection": 90.0, "weathercode": 0, "time": "2025-08-04T12:00" } }"#,
        )
        .create();

    let cw = weather::fetch_current_weather_from(52.5, 13.4, &server.url()).unwrap();

    // Rohdaten wie zuvor
    assert_eq!(cw.temperature, 20.0);
    assert_eq!(cw.windspeed, 5.0);
    assert_eq!(cw.winddirection_deg, 90.0);
    assert_eq!(cw.weathercode, 0);

    // Kompass-Funktion (16er-Rose); 90° => "E"
    assert_eq!(weather::deg_to_compass(cw.winddirection_deg), "E");

    // NEU: angereicherte Sicht prüfen
    let view = cw.to_view();
    assert_eq!(view.winddirection, "E");
    assert_eq!(view.winddirection_deg, 90.0);

    m.assert();
}

#[test]
fn test_solar_position() {
    let dt = DateTime::parse_from_rfc3339("2025-08-04T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    let pos = sun::solar_position(52.5, 13.4, dt);

    // numerisch wie gehabt (Toleranz lassen, suncalc kann minimal driften)
    assert!((pos.azimuth - 199.25).abs() < 0.5);
    assert!((pos.elevation - 53.39).abs() < 0.5);

    // NEU: Kompass-View (199° ≈ SSW im 16er-Compass)
    let view = pos.to_view();
    assert_eq!(view.azimuth_compass, "SSW");
}

#[test]
fn test_sky_report_consistency() {
    let dt = DateTime::parse_from_rfc3339("2025-08-04T15:00:00Z")
        .unwrap()
        .with_timezone(&Utc);

    let weather_ref = weather::fetch_current_weather(52.5, 13.4).unwrap();
    let report = sky::report(52.5, 13.4, Some(dt)).unwrap();

    // Vergleiche mit dem Referenzwert aus weather::
    assert_eq!(report.weather.temperature, weather_ref.temperature);
    assert_eq!(report.weather.windspeed, weather_ref.windspeed);
    assert_eq!(report.weather.winddirection_deg, weather_ref.winddirection_deg);
    assert_eq!(report.weather.winddirection, weather::deg_to_compass(weather_ref.winddirection_deg));
    assert_eq!(report.weather.weathercode, weather_ref.weathercode);
    assert_eq!(report.weather.time, weather_ref.time);
}
