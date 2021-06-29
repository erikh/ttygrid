#![allow(dead_code)]

use std::{cell::RefCell, fmt, rc::Rc, usize::MAX};

type SafeGridHeader = Rc<RefCell<GridHeader>>;

#[derive(Clone)]
pub struct HeaderList(Vec<SafeGridHeader>);

impl Default for HeaderList {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl HeaderList {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Ord)]
pub struct GridHeader {
    index: Option<usize>,
    text: &'static str,
    min_size: Option<usize>,
    max_pad: Option<usize>,
    priority: usize,
    max_len: Option<usize>,
}

impl Default for GridHeader {
    fn default() -> Self {
        Self {
            index: None,
            text: "",
            min_size: None,
            max_pad: Some(4),
            priority: 0,
            max_len: None,
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

impl fmt::Display for HeaderList {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for header in self.0.clone() {
            write!(
                formatter,
                "{:<width$}",
                header.borrow().text,
                width = header
                    .borrow()
                    .max_len
                    .unwrap_or(header.borrow().text.len() + 2)
            )?
        }
        Ok(())
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
    max_len: Option<usize>,
}

impl GridItem {
    pub fn new(header: SafeGridHeader, contents: String) -> Self {
        Self {
            header,
            contents,
            max_len: None,
        }
    }

    fn len(&self) -> usize {
        self.contents.len() + 1 // right padding
    }

    fn set_max_len(&mut self, max_len: usize) {
        self.max_len = Some(max_len)
    }
}

impl fmt::Display for GridItem {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{:<max_len$}",
            self.contents,
            max_len = self.max_len.unwrap_or(self.len())
        )
    }
}

#[derive(Clone)]
pub struct TTYGrid {
    headers: HeaderList,
    selected: HeaderList,
    lines: Vec<GridLine>,
}

impl TTYGrid {
    pub fn new(headers: Vec<SafeGridHeader>) -> Self {
        Self {
            selected: HeaderList(headers.clone()),
            headers: HeaderList(headers),
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
        self.selected.0.push(header)
    }

    pub fn is_selected(&self, header: SafeGridHeader) -> bool {
        self.selected.0.contains(&header)
    }

    pub fn select_all_headers(&mut self) {
        let mut idx = 0;
        for header in self.headers.0.clone() {
            let header = header.clone();
            self.select(header, idx);
            idx += 1;
        }
    }

    pub fn deselect_all_headers(&mut self) {
        self.selected.0.clear()
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

        let mut cached_columns = Vec::new();

        let mut idx = 0;
        for header in self.headers.0.iter_mut() {
            header.borrow_mut().max_len = Some(len_map.max_len_for_column(header.clone()));
            cached_columns.insert(idx, header.borrow().max_len);
            idx += 1;
        }

        for line in self.lines.iter_mut() {
            let mut idx = 0;
            for mut item in line.0.iter_mut() {
                if cached_columns.get(idx).is_some() {
                    item.max_len = *cached_columns.get(idx).clone().unwrap();
                }
                idx += 1;
            }
        }

        let mut len = self.headers.0.len();

        while len > 0 {
            let mut headers = HeaderList::new();
            for header in self.headers.0.iter().take(len) {
                headers.0.push(header.clone())
            }

            eprintln!("headers: {:?}, len: {}", headers.0, headers.0.len());
            let mut max_len = len_map.max_len_for_headers(headers.clone());

            while max_len > w {
                let mut new_headers = headers.clone();
                let mut lowest_prio_index = MAX;
                let mut to_remove = None;
                let mut idx = new_headers.0.len();

                for header in new_headers.0.iter().rev() {
                    idx -= 1;
                    let priority = header.borrow().priority;
                    // we expect that the users will let stuff fall off to the right, so here we
                    // are optimizing for that WRT priority
                    if priority < lowest_prio_index {
                        eprintln!("priority: {}", priority);
                        to_remove = Some(idx);
                        lowest_prio_index = priority;
                    }

                    eprintln!("idx: {}", idx);
                }

                if to_remove.is_some() {
                    eprintln!(
                        "removing: {}, name: {}",
                        to_remove.unwrap(),
                        new_headers.0.get(to_remove.unwrap()).unwrap().borrow().text
                    );
                    new_headers.0.remove(to_remove.unwrap());
                    max_len = len_map.max_len_for_headers(new_headers.clone());
                    headers = new_headers;
                } else {
                    break;
                }
            }

            let index = headers.0.iter().fold(0, |acc, x| acc + x.borrow().priority);
            prio_map.push((index, (headers, max_len)));

            len -= 1;
        }

        prio_map.sort_by(|(lprio, (_, llen)), (rprio, (_, rlen))| {
            lprio.cmp(&rprio).then(llen.cmp(rlen))
        });

        let mut max_headers: Option<HeaderList> = None;
        let mut last_len = 0;

        for (_, (headers, max_len)) in prio_map.iter().rev() {
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
        for header in max_headers.unwrap().0 {
            self.select(header, idx);
            idx += 1;
        }

        eprintln!(
            "selected {:?}",
            self.selected
                .0
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
        writeln!(formatter, "{}", self.selected)?;

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
        self.0
            .iter()
            .fold(0, |acc, x| acc + x.max_len.unwrap_or(x.len()))
    }

    fn selected(&self, grid: &TTYGrid) -> Self {
        let mut ret = Vec::new();
        for item in self.0.iter() {
            eprintln!("{} checked", RefCell::borrow(&item.header).text);
            if grid.is_selected(item.header.clone()) {
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
            write!(formatter, "{}", contents)?
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

    fn max_len_for_column(&self, header: SafeGridHeader) -> usize {
        let mut max_len = 0;
        for line in self.0.clone() {
            let found = line
                .iter()
                .find(|i| RefCell::borrow(&i.0).eq(&header.borrow()));

            if found.is_none() {
                // FIXME make this Result<>
                panic!("could not find pre-existing column header in line");
            }

            if max_len < found.unwrap().1 {
                max_len = found.unwrap().1
            }
        }

        max_len + header.borrow().max_pad.unwrap_or(0) + 2
    }

    fn max_len_for_headers(&mut self, headers: HeaderList) -> usize {
        self.0
            .iter()
            .map(|line| {
                line.iter()
                    .filter_map(|(header, len)| {
                        // wtf is my life anyway
                        if headers.0.contains(header) {
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
