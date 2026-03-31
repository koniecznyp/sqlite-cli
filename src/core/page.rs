#[derive(Debug)]
pub enum PageType {
    TableLeaf,
    TableInterior,
}

pub struct PageHeader {
    pub page_type: PageType,
    pub size: usize,
    pub cell_count: u16,
    pub right_most_pointer: Option<u32>,
}

pub struct Page {
    pub header: PageHeader,
    pub cells: Vec<Cell>,
}

impl Page {
    pub fn get_cell_count(&self) -> usize {
        self.header.cell_count as usize
    }
}

#[derive(Debug, PartialEq)]
pub enum Cell {
    TableLeaf { payload: Vec<u8> },
    TableInterior { left_child_page: u32 },
}
