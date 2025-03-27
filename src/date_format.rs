use chrono::DateTime;
use chrono::Datelike;
use chrono::Local;

/// Format a date in French.
pub(crate) fn format_date_in_french(date: DateTime<Local>) -> String {
    let days = [
        "dimanche", "lundi", "mardi", "mercredi", "jeudi", "vendredi", "samedi",
    ];
    let months = [
        "janvier",
        "février",
        "mars",
        "avril",
        "mai",
        "juin",
        "juillet",
        "août",
        "septembre",
        "octobre",
        "novembre",
        "décembre",
    ];

    let day_of_week = days[date.weekday().num_days_from_sunday() as usize];
    let day = date.day();
    let month = months[(date.month() - 1) as usize];
    let year = date.year();

    format!("{day_of_week} {day} {month} {year}")
}
