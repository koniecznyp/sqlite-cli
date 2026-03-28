pub enum PageType {
    TableLeaf,
    TableInterior
}

#[allow(dead_code)]
pub struct PageHeader {
    pub page_type: PageType,
    pub size: usize,
    pub cell_count: u16
}

pub struct Page {
    pub header: PageHeader,
    pub cells: Vec<Cell>
}

impl Page {
    pub fn get_cell_count(&self) -> usize {
        self.header.cell_count as usize
    }
}

pub struct Cell {
    pub payload: Vec<u8>
}