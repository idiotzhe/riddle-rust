use chrono::{NaiveDateTime};
use std::net::UdpSocket;

pub fn get_local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

pub fn get_time_range_display(start: NaiveDateTime, end: NaiveDateTime) -> String {
    if start.date() == end.date() {
        format!("<p>{}</p><p>{}~{}</p>", 
            start.format("%Y/%m/%d"), 
            start.format("%H:%M"), 
            end.format("%H:%M"))
    } else {
        format!("<p>{}~</p><p> {}</p>", 
            start.format("%m/%d %H:%M"), 
            end.format("%m/%d %H:%M"))
    }
}
