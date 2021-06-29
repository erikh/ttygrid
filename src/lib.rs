#![allow(dead_code)]

use std::{cell::RefCell, fmt, rc::Rc};

type SafeGridHeader = Rc<RefCell<GridHeader>>;
type HeaderList = Vec<SafeGridHeader>;

#[derive(Clone, PartialEq, Eq, Debug, Ord)]
pub struct GridHeader {
    index: Option<usize>,
    text: &'static str,
    min_size: Option<usize>,
    max_pad: Option<usize>,
    priority: usize,
}

impl Default for GridHeader {
    fn default() -> Self {
        Self {
            index: None,
            text: "",
            min_size: None,
            max_pad: None,
            priority: 0,
        }
    }
}

impl PartialOrd for GridHeader {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.index.is_some() {
            Some(
                self.priority
                    .cmp(&other.priority)
                    .then(self.index.cmp(&other.index)),
            )
        } else {
            Some(self.priority.cmp(&other.priority))
        }
    }
}

impl fmt::Display for GridHeader {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ", self.text)
    }
}

impl GridHeader {
    pub fn set_text(mut self, text: &'static str) -> Self {
        self.text = text;
        self
    }

    pub fn set_priority(mut self, priority: usize) -> Self {
        self.priority = priority;
        self
    }

    pub fn set_index(&mut self, idx: usize) {
        self.index = Some(idx);
    }
}

#[derive(Clone)]
pub struct GridItem {
    header: SafeGridHeader,
    contents: String,
}

impl GridItem {
    pub fn new(header: SafeGridHeader, contents: String) -> Self {
        Self { header, contents }
    }

    fn len(&self) -> usize {
        self.contents.len() + 1 // right padding
    }
}

#[derive(Clone)]
pub struct TTYGrid {
    headers: HeaderList,
    selected: HeaderList,
    lines: Vec<GridLine>,
}

impl TTYGrid {
    pub fn new(headers: HeaderList) -> Self {
        Self {
            selected: Vec::new(),
            headers,
            lines: Vec::new(),
        }
    }

    pub fn add_line(&mut self, item: GridLine) {
        self.lines.push(item)
    }

    pub fn clear_lines(&mut self) {
        self.lines.clear()
    }

    pub fn select(&mut self, header: SafeGridHeader, idx: usize) {
        // update index (still an issue)
        RefCell::borrow_mut(&header).set_index(idx);
        self.selected.push(header)
    }

    pub fn is_selected(&self, header: SafeGridHeader) -> bool {
        self.selected.contains(&header)
    }

    pub fn select_all_headers(&mut self) {
        let mut idx = 0;
        for header in self.headers.clone() {
            let header = header.clone();
            self.select(header, idx);
            idx += 1;
        }
    }

    pub fn deselect_all_headers(&mut self) {
        self.selected.clear()
    }

    fn determine_headers(&mut self) -> Result<(), std::io::Error> {
        let (w, _) = termion::terminal_size()?;
        let w = w as usize;

        let mut len_map = LengthMapper::default();
        let last = len_map.map_lines(self.lines.clone());

        eprintln!("{:?}", len_map);
        eprintln!("w: {}, last: {}", w, last);

        if last <= w {
            eprintln!("select all");
            self.select_all_headers();
            return Ok(());
        }

        eprintln!("priority select");

        let mut prio_map: Vec<(usize, (HeaderList, usize))> = Vec::new();
        self.deselect_all_headers();
        let mut priority = self.headers.clone();
        priority.sort();
        eprintln!("{:?}", priority);

        let mut len = priority.len();

        while len > 0 {
            let mut headers = HeaderList::new();
            for header in priority.iter().take(len) {
                headers.push(header.clone())
            }

            eprintln!("headers: {:?}, len: {}", headers, headers.len());
            let mut max_len = len_map.max_len_for_headers(headers.clone());

            while max_len > w {
                let mut new_headers = headers.clone();
                let mut lowest_prio_index = 0;
                let mut to_remove = None;
                let mut idx = new_headers.len();

                for header in new_headers.iter().rev() {
                    let priority = RefCell::borrow(header).priority;
                    // we expect that the users will let stuff fall off to the right, so here we are optimizing for that WRT priority
                    if priority < lowest_prio_index {
                        to_remove = Some(idx);
                        lowest_prio_index = priority;
                    }

                    idx -= 1;
                }

                if to_remove.is_some() {
                    new_headers.remove(to_remove.unwrap());
                    max_len = len_map.max_len_for_headers(new_headers);
                } else {
                    break;
                }
            }

            let index = headers
                .iter()
                .fold(0, |acc, x| acc + RefCell::borrow(x).priority);
            prio_map.push((index, (headers, max_len)));

            len -= 1;
        }

        eprintln!("{:?}", prio_map);
        prio_map.sort_by(|(lprio, (_, llen)), (rprio, (_, rlen))| {
            lprio.cmp(&rprio).then(llen.cmp(rlen))
        });

        let mut max_headers: Option<HeaderList> = None;
        let mut last_len = 0;

        for (_, (headers, max_len)) in prio_map.iter() {
            if *max_len <= w && *max_len > last_len {
                max_headers = Some(headers.clone());
                last_len = *max_len;
            }
        }

        if max_headers.is_none() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "your terminal is too small",
            ));
        }

        let mut idx = 0;
        for header in max_headers.unwrap() {
            self.select(header, idx);
            idx += 1;
        }

        eprintln!(
            "selected {:?}",
            self.selected
                .iter()
                .map(|h| RefCell::borrow(h).text)
                .collect::<Vec<&str>>()
        );

        Ok(())
    }

    pub fn display(&mut self) -> Result<String, std::io::Error> {
        self.determine_headers()?;
        Ok(format!("{}", self))
    }
}

impl fmt::Display for TTYGrid {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        for header in self.selected.clone() {
            write!(formatter, "{}", RefCell::borrow(&header))?
        }

        writeln!(formatter)?;

        for line in self.lines.clone() {
            writeln!(formatter, "{}", line.selected(self))?
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct GridLine(pub Vec<GridItem>);

impl GridLine {
    fn len(&self) -> usize {
        self.0.iter().fold(0, |acc, x| acc + x.len())
    }

    fn selected(&self, grid: &TTYGrid) -> Self {
        let mut ret = Vec::new();
        for item in self.0.iter() {
            eprintln!("{} checked", RefCell::borrow(&item.header).text);
            if grid.is_selected(item.header.clone()) {
                eprintln!("{} selected", RefCell::borrow(&item.header));
                ret.push(item.clone())
            }
        }

        GridLine(ret)
    }
}

impl Default for GridLine {
    fn default() -> Self {
        GridLine(Vec::new())
    }
}

impl fmt::Display for GridLine {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        for contents in self.0.clone() {
            write!(formatter, "{} ", contents.contents)?
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct LengthMapper(Vec<Vec<(SafeGridHeader, usize)>>);

impl Default for LengthMapper {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl LengthMapper {
    fn map_lines(&mut self, lines: Vec<GridLine>) -> usize {
        let mut last = 0;
        for line in lines.clone() {
            let len = self.0.len();
            let mut line_len = 0;
            self.0.push(Vec::new()); // now len is equal to index
            for item in line.0 {
                self.0
                    .get_mut(len)
                    .unwrap()
                    .push((item.header.clone(), item.len()));
                line_len += item.len();
            }

            if last < line_len {
                last = line_len
            }
        }

        last
    }

    fn max_len_for_headers(&mut self, headers: HeaderList) -> usize {
        self.0
            .iter()
            .map(|line| {
                line.iter()
                    .filter_map(|(header, len)| {
                        // wtf is my life anyway
                        if headers.contains(header) {
                            Some((header, len))
                        } else {
                            None
                        }
                    })
                    .fold(0, |acc, (_, x)| acc + x)
            })
            .max()
            .unwrap()
    }
}
