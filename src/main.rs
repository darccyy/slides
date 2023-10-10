use std::fs;

fn main() {
    let file = fs::read_to_string("example.md").unwrap();

    let parsed = parse_file(&file);
    // println!("{:#?}", parsed);

    let slides = make_slides(parsed);
    println!("{:#?}", slides);
}

#[derive(Debug)]
enum Slide {
    Title { header: String },
    Normal { header: String, body: Vec<String> },
}

fn make_slides(pieces: Vec<Piece>) -> Vec<Slide> {
    let mut slides: Vec<Slide> = Vec::new();
    let mut current: Option<(String, Vec<String>)> = None;

    for Piece(kind, text) in pieces {
        match kind {
            PieceKind::Header { level: 0 } => {
                if let Some((header, body)) = current {
                    slides.push(Slide::Normal { header, body });
                }
                current = None;
                slides.push(Slide::Title {
                    header: text.join(" "),
                })
            }
            PieceKind::Header { level: 1 } => {
                if let Some((header, body)) = current {
                    slides.push(Slide::Normal { header, body });
                }
                current = Some((text.join(" "), Vec::new()));
            }
            PieceKind::Header { level } => {
                let mut text = text
                    .into_iter()
                    .map(|line| format!("!!{}!! ({level})", line))
                    .collect();
                match &mut current {
                    Some((_, body)) => {
                        body.append(&mut text);
                    }
                    None => current = Some((String::new(), text)),
                }
            }
            PieceKind::Paragraph => {
                let mut text = text;
                match &mut current {
                    Some((_, body)) => {
                        body.append(&mut text);
                    }
                    None => current = Some((String::new(), text)),
                }
            }
            PieceKind::Quote => {
                let mut text = text
                    .into_iter()
                    .map(|line| format!("[[{}]]", line))
                    .collect();
                match &mut current {
                    Some((_, body)) => {
                        body.append(&mut text);
                    }
                    None => current = Some((String::new(), text)),
                }
            }
            _ => (),
        };
    }

    if let Some((header, body)) = current {
        slides.push(Slide::Normal { header, body });
    }

    slides
}

#[derive(Debug)]
enum PieceKind {
    Paragraph,
    Header { level: usize },
    ListItem { ordered: bool, depth: usize },
    Quote,
}

#[derive(Debug)]
struct Piece(PieceKind, Vec<String>);

fn parse_file(file: &str) -> Vec<Piece> {
    let mut parsed = Vec::new();
    let mut current: Option<Piece> = None;

    for line in file.lines() {
        let mut words = line.trim().split(" ");

        // Get first word
        let Some(word) = words.next().filter(|word| !word.is_empty()) else {
            // Blank line
            if let Some(current) = current {
                parsed.push(current);
            }
            current = None;
            continue;
        };

        let rest = words.collect::<Vec<_>>().join(" ");

        // Header
        if word.chars().all(|ch| ch == '#') {
            current = None;
            parsed.push(Piece(
                PieceKind::Header {
                    level: word.len() - 1,
                },
                vec![rest],
            ));
            continue;
        }

        // Quote
        if word == ">" {
            match &mut current {
                Some(Piece(PieceKind::Quote, lines)) => lines.push(rest),
                _ => {
                    if let Some(current) = current {
                        parsed.push(current);
                    }
                    current = Some(Piece(PieceKind::Quote, vec![rest]))
                }
            }
            continue;
        }

        const INDENT_SIZE: usize = 4;

        fn get_list_depth(line: &str) -> usize {
            line.chars().position(|ch| ch != ' ').unwrap_or(0) / INDENT_SIZE
        }

        // Unordered list
        if word == "-" {
            current = None;
            parsed.push(Piece(
                PieceKind::ListItem {
                    ordered: false,
                    depth: get_list_depth(line),
                },
                vec![rest],
            ));
            continue;
        }

        fn is_ordered_ident(word: &str) -> bool {
            let mut chars = word.chars();
            if !chars.next_back().is_some_and(|ch| ch == '.') {
                return false;
            }
            chars.as_str().parse::<usize>().is_ok()
        }

        // Ordered list
        if is_ordered_ident(word) {
            current = None;
            parsed.push(Piece(
                PieceKind::ListItem {
                    ordered: true,
                    depth: get_list_depth(line),
                },
                vec![rest],
            ));
            continue;
        }

        // Leave this here so we don't forget a `continue` statement above
        drop(rest);

        // Paragraph
        match &mut current {
            Some(Piece(PieceKind::Paragraph, lines)) => lines.push(line.to_string()),
            _ => {
                if let Some(current) = current {
                    parsed.push(current);
                }
                current = Some(Piece(PieceKind::Paragraph, vec![line.to_string()]))
            }
        }
    }

    if let Some(current) = current {
        parsed.push(current);
    }

    parsed
}
