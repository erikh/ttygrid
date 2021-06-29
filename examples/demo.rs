use std::cell::RefCell;
use std::env::args;
use std::rc::Rc;

const DEFAULT_MAX_LEN: u8 = 30;
const DEFAULT_MIN_LEN: u8 = 10;
const DEFAULT_ROWS: usize = 10;

use ttygrid::{GridHeader, GridItem, GridLine, TTYGrid};

// this is a handy random string generator I use in a few spots.
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

fn main() -> Result<(), anyhow::Error> {
    // provide defaults if no or invalid arguments are provided. basically, display a table no
    // matter what. max_len and min_len are provided as the first and second arguments respectively
    // to the program, and control the size of the randomly generated strings in the table.
    // The row count is the third argument, and controls how many lines to display in the table.
    // The table is a one shot rendering (as opposed to iteratively calculating and flushing) so it
    // can be a good perf test to use large numbers.

    let max_len = args()
        .nth(1)
        .unwrap_or(DEFAULT_MAX_LEN.to_string())
        .parse()
        .unwrap_or(DEFAULT_MAX_LEN);

    let min_len = args()
        .nth(2)
        .unwrap_or(DEFAULT_MIN_LEN.to_string())
        .parse()
        .unwrap_or(DEFAULT_MIN_LEN);

    let rows = args()
        .nth(3)
        .unwrap_or(DEFAULT_ROWS.to_string())
        .parse()
        .unwrap_or(DEFAULT_ROWS);

    // establish the headers, which we will use to bind tabular data to later. Each header has a
    // few properties:
    //
    // - text: this is the display text in the header. Header text is also used to manage column
    //   widths / padding
    //
    // - priority: this is what the engine uses to determine what columns to /keep/ in the event
    //   the whole line cannot be displayed. Higher number is higher priority.
    //
    // - max_pad: the padding value to apply when calculating line lengths. This is always greater
    //   than 2, even for zero-length strings.
    //
    // The whole thing uses the rust "builder" pattern where you compose your parameters by calling
    // set functions on a chainable object. The result after all the function calls is what is
    // used.
    let header_lineno = Rc::new(RefCell::new(
        GridHeader::default().set_text("line").set_priority(0),
    ));
    let header_one = Rc::new(RefCell::new(
        GridHeader::default().set_text("one").set_priority(1),
    ));
    let header_two = Rc::new(RefCell::new(
        GridHeader::default().set_text("two").set_priority(2),
    ));
    let header_three = Rc::new(RefCell::new(
        GridHeader::default().set_text("three").set_priority(3),
    ));
    let header_four = Rc::new(RefCell::new(
        GridHeader::default().set_text("four").set_priority(4),
    ));
    let header_five = Rc::new(RefCell::new(
        GridHeader::default().set_text("five").set_priority(5),
    ));
    let mut grid = TTYGrid::new(vec![
        header_lineno.clone(),
        header_one.clone(),
        header_two.clone(),
        header_three.clone(),
        header_four.clone(),
        header_five.clone(),
    ]);

    // for each line, associate data with each header in a GridLine vector; this data must include
    // the header and the content. Headers *must* be the above composed objects; they cannot be
    // re-composed. Soon, macros will help ferry this process along.
    for lineno in 0..rows {
        grid.add_line(GridLine(vec![
            GridItem::new(header_lineno.clone(), format!("{}", lineno)),
            GridItem::new(
                header_one.clone(),
                format!("{}", randstring(rand::random::<u8>() % max_len + min_len)),
            ),
            GridItem::new(
                header_two.clone(),
                format!("{}", randstring(rand::random::<u8>() % max_len + min_len)),
            ),
            GridItem::new(
                header_three.clone(),
                format!("{}", randstring(rand::random::<u8>() % max_len + min_len)),
            ),
            GridItem::new(
                header_four.clone(),
                format!("{}", randstring(rand::random::<u8>() % max_len + min_len)),
            ),
            GridItem::new(
                header_five.clone(),
                format!("{}", randstring(rand::random::<u8>() % max_len + min_len)),
            ),
        ]));
    }

    // finally, display the content. This is the expensive part as calculation is done, a string is
    // generated to memory, and then output.
    //
    // You might be asking yourself why we don't write to the FD/etc directly, to which I say it's
    // easier to test a string, and that nobody wants 10000 lines of tabular data sized for a
    // tty-only scenario. All that said, the algorithm is a big-O nightmare but the results are
    // fairly good.

    println!("{}", grid.display()?);
    Ok(())
}
