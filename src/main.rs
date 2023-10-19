use std::fs;

fn main() {
    let file = fs::read_to_string("example.md").unwrap();

    let parsed = parse_file(&file);
    // println!("{:#?}", parsed);

    let width = 100;
    let slides = make_slides(parsed, width);
    // println!("{:#?}", slides);

    for slide in slides {
        println!();
        println!("{}", "-".repeat(width));
        match slide {
            Slide::Title { header } => println!("{}", header),
            Slide::Normal { header, body } => {
                println!("{}", header);
                let body = body.join("\n");
                println!("{}", body);
            }
        }
        println!("{}", "-".repeat(width));
    }
}

#[derive(Debug)]
enum Slide {
    Title { header: String },
    Normal { header: String, body: Vec<String> },
}

fn centered_padding(text: &str, width: usize) -> String {
    " ".repeat(width.checked_sub(text.chars().count()).unwrap_or(0) / 2)
}

fn wrap_words(text: &str, width: usize) -> String {
    let mut result = String::new();
    let mut current_line = String::new();
    let mut line_len = 0;

    for word in text.split_whitespace() {
        let word_len = word.chars().count();

        if line_len + word_len + 1 <= width {
            if !current_line.is_empty() {
                current_line.push(' ');
                line_len += 1;
            }
            current_line += word;
            line_len += word_len;
        } else {
            if !result.is_empty() {
                result.push('\n');
            }
            result += &current_line;
            current_line.clear();
            current_line += word;
            line_len = word_len;
        }
    }

    if !result.is_empty() {
        result.push('\n');
    }
    result += &current_line;

    result
}

fn make_slides(pieces: Vec<Piece>, width: usize) -> Vec<Slide> {
    let mut slides: Vec<Slide> = Vec::new();
    let mut current: Option<(String, Vec<String>)> = None;

    let mut ordered_item_indexes = Vec::<usize>::new();

    for Piece(kind, text) in pieces {
        match kind {
            PieceKind::Header { level: 0 } => {
                if let Some((header, body)) = current {
                    slides.push(Slide::Normal { header, body });
                }
                current = None;
                let text = text.join(" ");
                let padding = centered_padding(&text, width);
                slides.push(Slide::Title {
                    header: format!("{padding}\x1b[1;4m{text}\x1b[0m"),
                })
            }
            PieceKind::Header { level: 1 } => {
                if let Some((header, body)) = current {
                    slides.push(Slide::Normal { header, body });
                }
                let text = text.join(" ");
                let padding = centered_padding(&text, width);
                let text = format!("{padding}\x1b[1;4m{text}\x1b[0m");
                current = Some((text, Vec::new()));
            }
            PieceKind::Header { level: _level } => {
                let text = text.join(" ");
                let padding = centered_padding(&text, width - 6);
                let text = format!("{padding}\x1b[2m—— \x1b[0;3m{text}\x1b[0;2m ——\x1b[0m");
                match &mut current {
                    Some((_, body)) => {
                        body.push(text);
                    }
                    None => current = Some((String::new(), vec![text])),
                }
            }
            PieceKind::Paragraph => {
                let mut text = text
                    .into_iter()
                    .map(|line| wrap_words(&line, width))
                    .collect();
                match &mut current {
                    Some((_, body)) => {
                        body.append(&mut text);
                    }
                    None => current = Some((String::new(), text)),
                }
            }
            PieceKind::Quote => {
                const LINE_H: char = '─';
                const LINE_V: char = '│';
                const LINE_TL: char = '┌';
                const LINE_TR: char = '┐';
                const LINE_BL: char = '└';
                const LINE_BR: char = '┘';
                const QUOTE_INDENT: usize = 2;

                let padding = " ".repeat(QUOTE_INDENT);
                let color = "\x1b[0;2m";

                let longest_line = text
                    .iter()
                    .map(|line| line.chars().count())
                    .max()
                    .unwrap_or(0)
                    .max(4);

                let mut text: Vec<_> = text
                    .into_iter()
                    .map(|line| {
                        let space = " ".repeat(longest_line - line.len());
                        format!(
                            "{color}{padding}{LINE_V}\x1b[0m  {line}{space}  {color}{LINE_V}\x1b[0m",
                        )
                    })
                    .collect();

                let line_middle = LINE_H.to_string().repeat(longest_line + 4);
                let line_top = format!("{color}{padding}{LINE_TL}{line_middle}{LINE_TR}\x1b[0m");
                let line_bottom = format!("{color}{padding}{LINE_BL}{line_middle}{LINE_BR}\x1b[0m");
                text.insert(0, line_top);
                text.push(line_bottom);

                match &mut current {
                    Some((_, body)) => {
                        body.append(&mut text);
                    }
                    None => current = Some((String::new(), text)),
                }
            }
            PieceKind::ListItem {
                ordered: false,
                depth,
            } => {
                let text = text.join(" ");
                let indent = "    ".repeat(depth);
                let symbol = UL_SYMBOLS.get(depth % UL_SYMBOLS.len()).unwrap();
                let new_width = width - indent.len() - 4;
                let text = wrap_words(&text, new_width).replace("\n", &format!("\n{indent}    "));
                let text = format!("  \x1b[2m{indent}{symbol}\x1b[0m {text}");
                match &mut current {
                    Some((_, body)) => {
                        body.push(text);
                    }
                    None => current = Some((String::new(), vec![text])),
                }
            }
            PieceKind::ListItem {
                ordered: true,
                depth,
            } => {
                let text = text.join(" ");
                let indent = "    ".repeat(depth);
                let number_int = match ordered_item_indexes.get(depth) {
                    Some(number) => number + 1,
                    None => {
                        ordered_item_indexes.insert(depth, 1);
                        1
                    }
                };
                let number = format_number(number_int, depth);
                let number_width = number.len() + 4;
                let new_width = width - indent.len() - number_width;
                let text = wrap_words(&text, new_width)
                    .replace("\n", &format!("\n{indent}{}", " ".repeat(number_width)));
                let text = format!("  \x1b[2m{indent}{number}.\x1b[0m {text}");
                match &mut current {
                    Some((_, body)) => {
                        body.push(text);
                    }
                    None => current = Some((String::new(), vec![text])),
                }
            }
        };
    }

    if let Some((header, body)) = current {
        slides.push(Slide::Normal { header, body });
    }

    slides
}

const UL_SYMBOLS: &[&str] = &["*", "-", "."];

fn format_number(number_int: usize, depth: usize) -> String {
    // make better!
    match depth % 3 {
        1 => ["a", "b", "c", "d"]
            .get(number_int)
            .unwrap_or(&"?")
            .to_string(),
        2 => ["i", "ii", "iii", "iv"]
            .get(number_int)
            .unwrap_or(&"?")
            .to_string(),
        _ => number_int.to_string(),
    }
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
            if let Some(current) = current {
                parsed.push(current);
            }
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
            if let Some(current) = current {
                parsed.push(current);
            }
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
