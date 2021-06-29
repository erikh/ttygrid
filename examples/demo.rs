use std::cell::RefCell;
use std::rc::Rc;

use ttygrid::{GridHeader, GridItem, GridLine, TTYGrid};

fn randstring(len: u8) -> String {
    (0..len)
        .map(|_| (rand::random::<u8>() % 26) + 'a' as u8)
        .map(|c| {
            if rand::random::<bool>() {
                (c as char).to_ascii_uppercase()
            } else {
                c as char
            }
            .to_string()
        })
        .collect::<Vec<String>>()
        .join("")
}

fn main() -> Result<(), std::io::Error> {
    let header_lineno = Rc::new(RefCell::new(
        GridHeader::default().set_text("line").set_priority(0),
    ));
    let header_one = Rc::new(RefCell::new(
        GridHeader::default().set_text("one").set_priority(1),
    ));
    let header_two = Rc::new(RefCell::new(
        GridHeader::default().set_text("two").set_priority(2),
    ));
    let header_extra = Rc::new(RefCell::new(
        GridHeader::default().set_text("extra").set_priority(3),
    ));
    let header_extra2 = Rc::new(RefCell::new(
        GridHeader::default().set_text("extra2").set_priority(4),
    ));
    let header_extra3 = Rc::new(RefCell::new(
        GridHeader::default().set_text("extra3").set_priority(5),
    ));
    let mut grid = TTYGrid::new(vec![
        header_lineno.clone(),
        header_one.clone(),
        header_two.clone(),
        header_extra.clone(),
        header_extra2.clone(),
        header_extra3.clone(),
    ]);

    for lineno in 0..10 {
        grid.add_line(GridLine(vec![
            GridItem::new(header_lineno.clone(), format!("{}", lineno)),
            GridItem::new(
                header_one.clone(),
                format!("{}", randstring(rand::random::<u8>() % 30 + 10)),
            ),
            GridItem::new(
                header_two.clone(),
                format!("{}", randstring(rand::random::<u8>() % 30 + 10)),
            ),
            GridItem::new(
                header_extra.clone(),
                format!("{}", randstring(rand::random::<u8>() % 30 + 10)),
            ),
            GridItem::new(
                header_extra2.clone(),
                format!("{}", randstring(rand::random::<u8>() % 30 + 10)),
            ),
            GridItem::new(
                header_extra3.clone(),
                format!("{}", randstring(rand::random::<u8>() % 30 + 10)),
            ),
        ]));
    }
    println!("{}", grid.display()?);
    Ok(())
}
