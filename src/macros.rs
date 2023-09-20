/// grid defines a [crate::TTYGrid] full of headers, which are associated with lines.
///
/// Each header is typically defined by the [crate::header!] macro, and [crate::add_line!] is used
/// to add content to the grid. Headers may have an optional priority; in the case where the line
/// is too long to be displayed on a terminal of that width, columns with a lower priority will be
/// removed to help the higher priority items fit.
///
/// Example:
/// ```
///    use ttygrid::{grid, add_line, header};
///    let mut grid = grid!(
///        header!("line"),     header!("one", 1),  header!("two", 2),
///        header!("three", 3), header!("four", 4), header!("five", 5)
///    ).unwrap();
///
///    add_line!(grid, "0", "1", "2", "3", "4", "5");
///
///    println!("{}", grid.display().unwrap());
/// ```
///
#[macro_export]
macro_rules! grid {
    ($($header:expr),*) => {
        {
            use $crate::TTYGrid;
            TTYGrid::new(vec![$($header.clone(),)*])
        }
    };
}

/// header defines a [crate::SafeGridHeader] for use with the [crate::TTYGrid].
///
/// It is variadic and composes of two current options:
///
/// - text by itself as the first position will yield a base header with the text set.
/// - a second parameter, optionally provided, will set the priority to a [usize]. This controls
///   display capabilities where the terminal width is too small to display all columns. See
///   [crate::grid!] for more.
///
/// Examples:
///
/// ```
///    use ttygrid::header;
///
///    assert_eq!(header!("header").borrow().text(), "header");
///
///    let priority_header = header!("header2", 10);
///    assert_eq!(priority_header.borrow().text(), "header2");
///    assert_eq!(priority_header.borrow().priority(), 10);
///
///    let name = "foo";
///    let priority = 20;
///    assert_eq!(header!(name, priority), header!("foo", 20));
/// ```
#[macro_export]
macro_rules! header {
    ($text:tt) => {{
        use std::cell::RefCell;
        use std::rc::Rc;
        use $crate::GridHeader;
        Rc::new(RefCell::new(GridHeader::default().set_text($text)))
    }};

    ($text:tt,$priority:tt) => {{
        use std::cell::RefCell;
        use std::rc::Rc;
        use $crate::GridHeader;
        Rc::new(RefCell::new(
            GridHeader::default()
                .set_text($text)
                .set_priority($priority),
        ))
    }};
}

/// add_line defines a [crate::GridLine] with [crate::GridItem]s attached.
///
/// The first element provided is the grid; and the rest are strings which correspond to headers
/// set to the grid, in order of appearance. A line **must** be equal to the number of headers,
/// otherwise this macro will yield [anyhow::Error].
///
/// Please see the [crate::grid!] example for more.
#[macro_export]
macro_rules! add_line {
    ($grid:expr, $($content:expr),*) => {
        {
            use anyhow::anyhow;
            use $crate::{GridLine, GridItem};
            let content = vec![$($content),*];

            if content.len() != $grid.headers().len() {
                Err(anyhow!("ttygrid panic: content items must equal the number of headers"))
            } else {
                $grid.add_line(GridLine(content.iter().enumerate().map(|(i, item)| {
                    GridItem::new($grid.headers().get(i).unwrap().clone(), item.to_string())
                }).collect()));

                Ok(())
            }
        }
    };
}
