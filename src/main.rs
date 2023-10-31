use regex::Regex;
use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::{accept, Message};

/// A WebSocket echo server

#[derive(Debug, Clone, Copy)]
struct Board {
    data: [[u8; 3]; 3],
    eval: u8,
    /*
     * 0 is undecided
     * 1 is draw
     * 4 is win for X
     * 5 is win for O
     */
}

fn main() {
    let var: Vec<Board> = vec![];
    let mutex = std::sync::Mutex::new(var);
    let arc = std::sync::Arc::new(mutex);
    let server = TcpListener::bind("127.0.0.1:9001").unwrap();
    for stream in server.incoming() {
        let arc = arc.clone();
        spawn(move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            let mut current = Board {
                data: [[0; 3]; 3],
                eval: 0,
            };
            let regex = Regex::new(r"(\d+) (\d+)").unwrap();
            loop {
                let msg = websocket.read().unwrap();
                if !(msg.is_binary() || msg.is_text()) {
                    continue;
                }
                let mut guard = arc.lock().unwrap();
                let coord: Vec<(usize, usize)> = regex
                    .captures_iter(msg.to_string().as_str())
                    .map(|caps| {
                        let (_, [x, y]) = caps.extract();
                        (x.parse::<usize>().unwrap(), y.parse::<usize>().unwrap())
                    })
                    .take(1)
                    .filter(|val| val.0 < 3 && val.1 < 3)
                    .collect();
                if coord.len() == 0 {
                    websocket.send(Message::text(",Invalid input")).unwrap();
                    continue;
                }
                let (x, y) = coord[0];
                let value = if current.data.iter().fold(0, |curr, row| {
                    curr + row
                        .iter()
                        .fold(0, |current, elem| current + if *elem != 0 { 1 } else { 0 })
                }) & 1
                    == 0
                {
                    2
                } else {
                    3
                };
                if !moveable(current.data, (x, y)) {
                    websocket.send(Message::text(",Illegal move")).unwrap();
                    continue;
                }
                let mut response = String::from("");
                if check_win(current.data, (x, y), value) {
                    response = format!(",{value} won\n");
                    current.eval = value + 2;
                }
                current.data[x][y] = value;
                let mut copy = current.data.clone();
                // for horizontal in current.data.iter_mut() {
                for i in 0..current.data.len() {
                    let horizontal = &mut current.data[i];
                    // for location in horizontal.iter_mut() {
                    for j in 0..horizontal.len() {
                        let location = &mut horizontal[j];
                        if !(*location == 2 || *location == 3) {
                            let loc = *location;
                            copy[i][j] = 5 - value;
                            eprintln!("{:?}", copy);
                            let index = (*guard)
                                .iter()
                                .position(|board| -> bool { board.data == copy });
                            copy[i][j] = match index {
                                Some(index) => {
                                    (*guard)[index].eval
                                }
                                None => loc,
                            };
                            *location = copy[i][j];
                        }
                        response = format!("{response}{location},")
                    }
                    response = format!("{response}\n");
                }
                response.pop();
                websocket.send(Message::text(response.to_string())).unwrap();
                (*guard).push(current.clone());
            }
        });
    }
}

fn moveable(data: [[u8; 3]; 3], (x, y): (usize, usize)) -> bool {
    !((data[x][y] == 2) || (data[x][y] == 3))
}

fn check_win(data: [[u8; 3]; 3], (x, y): (usize, usize), current: u8) -> bool {
    //horizontal detection
    let mut count: u8 = 0;
    for i in 1..=3 {
        if (x + i) == 3 {
            break;
        }
        if data[x + i][y] != current {
            break;
        }
        count += 1;
    }
    for i in 1..=3 {
        if x < i {
            break;
        }
        if data[x - i][y] != current {
            break;
        }
        count += 1;
    }
    if count >= 2 {
        return true;
    }

    //vertical detection
    count = 0;
    for i in 1..=3 {
        if (y + i) == 3 {
            break;
        }
        if data[x][y + i] != current {
            break;
        }
        count += 1;
    }
    for i in 1..=3 {
        if y < i {
            break;
        }
        if data[x][y - i] != current {
            break;
        }
        count += 1;
    }
    if count >= 2 {
        return true;
    }

    //diagonal down from top left detection
    count = 0;
    for i in 1..=3 {
        if (y + i) == 3 || (x + i) == 3 {
            break;
        }
        if data[x + i][y + i] != current {
            break;
        }
        count += 1;
    }
    for i in 1..=3 {
        if y < i || x < i {
            break;
        }
        if data[x - i][y - i] != current {
            break;
        }
        count += 1;
    }
    if count >= 2 {
        return true;
    }

    //diagonal up from bottom left detection
    count = 0;
    for i in 1..=3 {
        if (y + i) == 3 || x < i {
            break;
        }
        if data[x - i][y + i] != current {
            break;
        }
        count += 1;
    }
    for i in 1..=3 {
        if y < i || (x + i) == 3 {
            break;
        }
        if data[x + i][y - i] != current {
            break;
        }
        count += 1;
    }
    if count >= 2 {
        return true;
    }
    false
}
