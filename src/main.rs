use regex::Regex;
use std::cmp;
use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::{accept, Message};

/// A WebSocket echo server

#[derive(Debug, Clone, Copy)]
struct Board {
    data: [[u8; 3]; 3],
    depth: usize,
    eval: u8,
    /*
     * 0 is undecided
     * 1 is win for other player
     * 4 is draw
     * 5 is win for current player
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
                depth: 0,
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
                    curr + row.iter().fold(0, |current, elem| {
                        current + if *elem == 2 || *elem == 3 { 1 } else { 0 }
                    })
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
                    current.eval = 5;
                }
                current.data[x][y] = value;
                let copy = current.clone();

                for i in 0..current.data.len() {
                    let horizontal = &mut current.data[i];
                    for j in 0..horizontal.len() {
                        let location = &mut horizontal[j];
                        if !(*location == 2 || *location == 3) {
                            let mut localcopy = copy.clone();
                            localcopy.data[i][j] = 5 - value;
                            // println!("{:?}", localcopy.data);

                            if check_win(localcopy.data, (i, j), 5 - value) {
                                *location = 5;
                            } else {
                                println!("{:?}\n{:?}\n{value}", *guard, localcopy);
                                *location = flip(evaluate(&mut *guard, localcopy, 2, 5 - value));
                            }
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

fn evaluate(boards: &mut Vec<Board>, mut board: Board, depth: usize, curr: u8) -> u8 {
    let index = boards
        .iter()
        .position(|localboard| -> bool { localboard.data == board.data });
    if match index {
        Some(index) => boards[index].depth >= depth || boards[index].eval != 0,
        None => false,
    } {
        return match index {
            Some(index) => boards[index].eval,
            None => 0,
        };
    }
    if depth == 0 {
        return 0;
    }
    let copy = board.clone();
    let mut eval = 0;
    for i in 0..board.data.len() {
        let horizontal = &mut board.data[i];
        for j in 0..horizontal.len() {
            let location = &mut horizontal[j];
            if !(*location == 2 || *location == 3) {
                let mut localcopy = copy.clone();
                localcopy.data[i][j] = 5 - curr;
                if check_win(localcopy.data, (i, j), 5 - curr) {
                    return 5;
                }
                let evaluation = flip(evaluate(boards, localcopy, depth - 1, 5 - curr));
                eval = cmp::max(evaluation, eval);
            }
        }
    }
    eval
}
fn flip(eval: u8) -> u8 {
    match eval {
        1 => 5, //won
        4 => 4, //draw
        5 => 1, //lost
        _ => 0, //undecided
    }
}
fn moveable(data: [[u8; 3]; 3], (x, y): (usize, usize)) -> bool {
    !((data[x][y] == 2) || (data[x][y] == 3))
}

fn check_win(data: [[u8; 3]; 3], (x, y): (usize, usize), current: u8) -> bool {
    if data[x][y] == 5 - current {
        return false;
    }
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
