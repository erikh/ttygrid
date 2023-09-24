// demo program with tweakable args. This program creates a number of static columns, binds data of
// configurable and random lengths to the columns, and then attempts output. Based on your terminal
// width, you will see more or less data. Play with the options; they are these constants by
// default and supplied in order on the CLI.
const DEFAULT_MAX_LEN: u8 = 30;
const DEFAULT_MIN_LEN: u8 = 10;
const DEFAULT_ROWS: usize = 10;

use std::env::args;

use crossterm::style::{Color, Colors};
use ttygrid::{add_line, grid, header};

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

// small func to min/max random strings
fn rando(max_len: u8, min_len: u8) -> String {
    format!("{}", randstring(rand::random::<u8>() % max_len + min_len))
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
    // few properties (listed in order in the header!() macro, or use GridHeader directly with a
    // rust builder pattern):
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
    let header_lineno = header!("line");
    let header_one = header!("p3", 3);
    let header_two = header!("p1", 1);
    let header_three = header!("p4", 4);
    let header_four = header!("p5", 5);
    let header_five = header!("p2", 2);

    let mut g = grid!(
        header_lineno,
        header_one,
        header_two,
        header_three,
        header_four,
        header_five
    )?;

    // the add_line! macro lines up your content to the position in the grid. the rando() function
    // here just generates a random string.
    for lineno in 0..rows {
        add_line!(
            g,
            format!("{}", lineno),
            rando(max_len, min_len),
            rando(max_len, min_len),
            rando(max_len, min_len),
            rando(max_len, min_len),
            rando(max_len, min_len)
        )?
    }

    // spice it up with some colors. New in v0.3.0! You must use the write() API to get this
    // functionality.
    g.set_header_color(Colors::new(Color::DarkCyan, Color::Reset));
    g.set_delimiter_color(Colors::new(Color::Cyan, Color::Reset));
    g.set_primary_color(Colors::new(Color::White, Color::Reset));
    g.set_secondary_color(Colors::new(Color::Grey, Color::Reset));

    // finally, display the content.
    g.write(&mut std::io::stdout())?;
    Ok(())
}
