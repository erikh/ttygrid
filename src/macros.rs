#[macro_export]
macro_rules! grid {
    ($($header:expr),*) => {
        {
            use $crate::TTYGrid;
            TTYGrid::new(vec![$($header.clone(),)*])
        }
    };
}

#[macro_export]
macro_rules! header {
    ($text:literal) => {{
        use std::cell::RefCell;
        use std::rc::Rc;
        use $crate::GridHeader;
        Rc::new(RefCell::new(GridHeader::default().set_text($text)))
    }};

    ($text:literal,$priority:literal) => {{
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
                $grid.add_line(GridLine(content.iter().enumerate().map(|(i, item)| GridItem::new($grid.headers().get(i).unwrap().clone(), item.to_string())).collect()));
                Ok(())
            }
        }
    };
}
