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
    pub fn set_max_len(&mut self, len: usize) {
        self.max_len = Some(len)
    }

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
    width: usize,
}

impl TTYGrid {
    pub fn new(headers: Vec<SafeGridHeader>) -> Self {
        let (w, _) = termion::terminal_size().unwrap_or((80, 25));
        let width = w as usize;

        Self {
            selected: HeaderList::new(),
            headers: HeaderList(headers),
            lines: Vec::new(),
            width,
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
        header.borrow_mut().set_index(idx);
        self.selected.0.push(header)
    }

    pub fn is_selected(&self, header: SafeGridHeader) -> bool {
        self.selected.0.contains(&header)
    }

    pub fn select_all_headers(&mut self) {
        self.selected = self.headers.clone()
    }

    pub fn deselect_all_headers(&mut self) {
        self.selected.0.clear()
    }

    fn set_grid_max_len(&mut self, len_map: &LengthMapper) {
        let mut cached_columns = Vec::new();

        let mut idx = 0;
        for header in self.headers.0.iter_mut() {
            let max_len = len_map.max_len_for_column(&header.borrow());
            header.borrow_mut().set_max_len(max_len);
            cached_columns.insert(idx, header.borrow().max_len);
            idx += 1;
        }

        for line in self.lines.iter_mut() {
            let mut idx = 0;
            for item in line.0.iter_mut() {
                if cached_columns.get(idx).is_some() {
                    item.set_max_len(cached_columns.get(idx).clone().unwrap().unwrap());
                }
                idx += 1;
            }
        }
    }

    fn determine_headers(&mut self) -> Result<(), std::io::Error> {
        let mut len_map = LengthMapper::default();
        len_map.map_lines(self.lines.clone());

        self.set_grid_max_len(&len_map); // this has to happen before any return occurs
        let last = len_map.max_len_for_headers(self.headers.clone());

        if last <= self.width {
            self.select_all_headers();
            return Ok(());
        }

        let mut prio_map: Vec<(usize, (HeaderList, usize))> = Vec::new();
        self.deselect_all_headers();

        let mut len = self.headers.0.len();

        while len > 0 {
            let mut headers = HeaderList::new();
            for header in self.headers.0.iter().take(len) {
                headers.0.push(header.clone())
            }

            let mut max_len = len_map.max_len_for_headers(headers.clone());

            while max_len > self.width {
                let mut new_headers = headers.clone();
                let mut lowest_prio_index = MAX;
                let mut to_remove = None;
                let mut idx = 0;

                for header in new_headers.0.iter() {
                    let priority = header.borrow().priority;
                    if priority < lowest_prio_index {
                        to_remove = Some(idx);
                        lowest_prio_index = priority;
                    }
                    idx += 1;
                }

                if to_remove.is_some() {
                    new_headers.0.remove(to_remove.unwrap());
                    max_len = len_map.max_len_for_headers(new_headers.clone());
                    headers = new_headers;
                } else {
                    max_len = 0 // bury it
                }
            }

            let index = headers.0.iter().fold(0, |acc, x| acc + x.borrow().priority);
            prio_map.push((index, (headers, max_len)));
            len -= 1;
        }

        if prio_map.len() == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "your terminal is too small",
            ));
        }

        prio_map.sort_by(|(lprio, (_, llen)), (rprio, (_, rlen))| {
            lprio.cmp(&rprio).then(llen.cmp(rlen))
        });

        let (_, (max_headers, _)) = prio_map.iter().last().unwrap();

        let mut idx = 0;
        for header in max_headers.0.iter() {
            self.select(header.clone(), idx);
            idx += 1;
        }

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
        writeln!(formatter, "{:-<width$}", "-", width = self.width)?;

        for line in self.lines.clone() {
            writeln!(formatter, "{}", line.selected(self))?
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct GridLine(pub Vec<GridItem>);

impl GridLine {
    fn selected(&self, grid: &TTYGrid) -> Self {
        let mut ret = Vec::new();
        for item in self.0.iter() {
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
    fn map_lines(&mut self, lines: Vec<GridLine>) {
        for line in lines.clone() {
            let len = self.0.len();
            self.0.push(Vec::new()); // now len is equal to index
            for item in line.0 {
                self.0
                    .get_mut(len)
                    .unwrap()
                    .push((item.header.clone(), item.len()));
            }
        }
    }

    fn max_len_for_column(&self, header: &GridHeader) -> usize {
        let mut max_len = 0;
        for line in self.0.clone() {
            let found = line.iter().find(|i| i.0.borrow().eq(&header));

            if found.is_none() {
                // FIXME make this Result<>
                panic!("could not find pre-existing column header in line");
            }

            if max_len < found.unwrap().1 {
                max_len = found.unwrap().1
            }
        }

        max_len + header.max_pad.unwrap_or(0) + 2
    }

    fn max_len_for_headers(&mut self, headers: HeaderList) -> usize {
        headers
            .0
            .iter()
            .fold(0, |x, h| x + self.max_len_for_column(&h.clone().borrow()))
    }
}
